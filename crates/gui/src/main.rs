mod cli;

use clap::Parser;
use cli::Cli;
use eframe::egui;
use entities::report::Model as Report;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

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
    tx: Sender<ApiRequest>,
    rx: Receiver<ApiResponse>,
}

impl ResultGuideGui {
    fn new(cli: Cli) -> Self {
        let (req_tx, req_rx) = std::sync::mpsc::channel();
        let (resp_tx, resp_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let host_name = cli.host.unwrap_or("localhost".to_string());
            let port = cli.port.unwrap_or(80);
            let url = format!("http://{host_name}:{port}");
            loop {
                match req_rx.recv() {
                    Ok(ApiRequest::GetReports) => {
                        let command_url = format!("{url}/reports");
                        let body = reqwest::blocking::get(&command_url)
                            .unwrap()
                            .text()
                            .unwrap();
                        resp_tx.send(ApiResponse::Report(serde_json::from_str(&body).unwrap())).unwrap();
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            resp: None,
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
}

impl eframe::App for ResultGuideGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Result Guide");
            if ui.button("Get Reports").clicked() {
                self.tx.send(ApiRequest::GetReports).unwrap()
            }
            if let Ok(ApiResponse::Report(resp)) = self.rx.try_recv() {
                self.resp = Some(resp);
            }
            ui.label(match &self.resp {
                Some(resp) => format!("{resp:?}"),
                None => String::new(),
            });
        });
    }
}
