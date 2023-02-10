use eframe::egui;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Result Guide",
        options,
        Box::new(|_cc| Box::<ResultGuideGui>::default()),
    )
    .unwrap();
}

struct ResultGuideGui {
    resp: Option<String>,
    tx: Sender<ApiRequest>,
    rx: Receiver<ApiResponse>,
}

enum ApiRequest {
    GetReports,
}
enum ApiResponse {
    Raw(String),
}

impl Default for ResultGuideGui {
    fn default() -> Self {
        let (req_tx, req_rx) = std::sync::mpsc::channel();
        let (resp_tx, resp_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || loop {
            match req_rx.recv() {
                Ok(ApiRequest::GetReports) => {
                    let body = reqwest::blocking::get("https://www.rust-lang.org")
                        .unwrap()
                        .text()
                        .unwrap();
                    resp_tx.send(ApiResponse::Raw(body)).unwrap();
                }
                Err(_) => break,
            }
        });

        Self {
            resp: None,
            tx: req_tx,
            rx: resp_rx,
        }
    }
}

impl eframe::App for ResultGuideGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Result Guide");
            if ui.button("Get Reports").clicked() {
                self.tx.send(ApiRequest::GetReports).unwrap()
            }
            if let Ok(ApiResponse::Raw(resp)) = self.rx.try_recv() {
                self.resp = Some(resp);
            }
            ui.label(match &self.resp {
                Some(resp) => resp.clone(),
                None => String::new(),
            });
        });
    }
}
