mod cli;

use clap::Parser;
use cli::Cli;
use eframe::egui;
use entities::report::Model as Report;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Result Guide",
        options,
        Box::new(|_cc| Box::new(ResultGuideGui::new(Cli::parse()))),
    )
    .unwrap();
}

struct ResultGuideGui {
    resp: Option<Vec<Report>>,
    last_error: Option<eyre::Report>,
    tx: Sender<ApiRequest>,
    rx: Receiver<ApiResponse>,
}

impl ResultGuideGui {
    fn retrieve_reports(url: &str) -> eyre::Result<Vec<Report>> {
        let command_url = format!("{url}/reports");
        let response = reqwest::blocking::get(command_url)?;
        let body = response.text()?;
        Ok(serde_json::from_str(&body)?)
    }

    fn new(cli: Cli) -> Self {
        let (req_tx, req_rx) = std::sync::mpsc::channel();
        let (resp_tx, resp_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let host_name = cli.host.unwrap_or("localhost".to_string());
            let port = cli.port.unwrap_or(3000);
            let url = format!("http://{host_name}:{port}");
            loop {
                match req_rx.recv() {
                    Ok(ApiRequest::GetReports) => {
                        resp_tx
                            .send(match ResultGuideGui::retrieve_reports(&url) {
                                Ok(reports) => ApiResponse::Report(reports),
                                Err(e) => ApiResponse::Error(e),
                            })
                            .unwrap();
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            resp: None,
            last_error: None,
            tx: req_tx,
            rx: resp_rx,
        }
    }
}

enum ApiRequest {
    GetReports,
}
enum ApiResponse {
    Raw(String),
    Report(Vec<Report>),
    Error(eyre::Report),
}

impl eframe::App for ResultGuideGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Result Guide");
            if ui.button("Get Reports").clicked() {
                self.tx.send(ApiRequest::GetReports).unwrap()
            }
            match self.rx.try_recv() {
                Ok(ApiResponse::Report(resp)) => self.resp = Some(resp),
                Ok(ApiResponse::Error(e)) => self.last_error = Some(e),
                Err(TryRecvError::Disconnected) => panic!("waaah!"),
                _ => (),
            }
            ui.label(match &self.resp {
                Some(resp) => format!("{resp:?}"),
                None => String::new(),
            });
            ui.label(match &self.last_error {
                Some(e) => egui::RichText::new(e.to_string()).color(egui::Color32::RED),
                None => egui::RichText::new(""),
            });
        });
    }
}
