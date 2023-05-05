mod cli;

use clap::Parser;
use cli::Cli;
use eframe::egui;
use entities::report::Model as Report;
use entities::test_step::Model as TestStep;

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
    reports: Option<Vec<Report>>,
    test_steps: Option<Vec<TestStep>>,
    last_error: Option<eyre::Report>,
    tx: Sender<ApiRequest>,
    rx: Receiver<ApiResponse>,
}

impl ResultGuideGui {
    fn retrieve_reports(url: &str) -> eyre::Result<Vec<Report>> {
        let command_url = format!("{url}/reports");
        let response = reqwest::blocking::get(command_url)?.error_for_status()?;
        let body = response.text()?;
        Ok(serde_json::from_str(&body)?)
    }

    fn retrieve_test_steps(url: &str, report_id: i32) -> eyre::Result<Vec<TestStep>> {
        let command_url = format!("{url}/reports/{report_id}/test_steps");
        let response = reqwest::blocking::get(command_url)?.error_for_status()?;
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
                                Ok(reports) => ApiResponse::Reports(reports),
                                Err(e) => ApiResponse::Error(e),
                            })
                            .unwrap();
                    }
                    Ok(ApiRequest::GetTestSteps(report_id)) => {
                        resp_tx
                            .send(match ResultGuideGui::retrieve_test_steps(&url, report_id) {
                                Ok(test_steps) => ApiResponse::TestSteps(test_steps),
                                Err(e) => ApiResponse::Error(e),
                            })
                            .unwrap();
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            reports: None,
            test_steps: None,
            last_error: None,
            tx: req_tx,
            rx: resp_rx,
        }
    }
}

enum ApiRequest {
    GetReports,
    GetTestSteps(i32),
}
enum ApiResponse {
    Raw(String),
    Reports(Vec<Report>),
    TestSteps(Vec<TestStep>),
    Error(eyre::Report),
}

impl eframe::App for ResultGuideGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Result Guide");
            if ui.button("Get Reports").clicked() {
                self.tx.send(ApiRequest::GetReports).unwrap()
            }
            if ui.button("Get TestSteps of first shown Report").clicked() {
                if let Some([first, ..]) = &self.reports.as_deref() {
                    self.tx.send(ApiRequest::GetTestSteps(first.id)).unwrap()
                }
            }
            match self.rx.try_recv() {
                Ok(ApiResponse::Reports(resp)) => {
                    self.reports = Some(resp);
                    self.last_error = None;
                }
                Ok(ApiResponse::TestSteps(resp)) => {
                    self.test_steps = Some(resp);
                    self.last_error = None;
                }
                Ok(ApiResponse::Error(e)) => self.last_error = Some(e),
                Err(TryRecvError::Disconnected) => panic!("waaah!"),
                _ => (),
            }
            ui.label(match &self.reports {
                Some(resp) => format!("{resp:?}"),
                None => String::new(),
            });
            ui.label(match &self.test_steps {
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
