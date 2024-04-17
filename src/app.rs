use crate::{calc::to_buy, holdings::{StockSymbol, ShareValues, VanguardRebalance, VanguardHoldings, parse_csv_download}};
use futures::executor::block_on;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct VaporeApp {
    // Example stuff:
    age: u32,
    retirement_year: i32,
    brokerage_stock: u32,
    brokerage_account_num: u32,
    roth_account_num: u32,
    trad_account_num: u32,
    percent_stock: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    brokerage_cash_add: f32,
    brokerage_us_stock_add: f32,
    brokerage_int_stock_add: f32,
    brokerage_us_bond_add: f32,
    brokerage_int_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    roth_holdings: ShareValues,
    roth_us_stock_add: f32,
    roth_us_bond_add: f32,
    roth_int_stock_add: f32,
    roth_int_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    roth_cash_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    traditional_holdings: ShareValues,
    traditional_us_stock_add: f32,
    traditional_us_bond_add: f32,
    traditional_int_stock_add: f32,
    traditional_int_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    traditional_cash_add: f32,
    use_brokerage_retirement: bool,
    #[serde(skip)] // This how you opt-out of serialization of a field
    brokerage_holdings: ShareValues,
    #[serde(skip)] // This how you opt-out of serialization of a field
    rebalance: VanguardRebalance,
    #[serde(skip)] // This how you opt-out of serialization of a field
    vanguard_holdings: VanguardHoldings,
    #[serde(skip)] // This how you opt-out of serialization of a field
    stock_quotes: ShareValues
}

impl Default for VaporeApp {
    fn default() -> Self {
        Self {
            age: 0,
            retirement_year: 2025,
            brokerage_stock: 65,
            brokerage_account_num: 0,
            roth_account_num: 0,
            trad_account_num: 0,
            percent_stock: 0.0,
            brokerage_cash_add: 0.0,
            brokerage_us_stock_add: 0.0,
            brokerage_int_stock_add: 0.0,
            brokerage_us_bond_add: 0.0,
            brokerage_int_bond_add: 0.0,
            roth_holdings: ShareValues::new(),
            roth_us_stock_add: 0.0,
            roth_us_bond_add: 0.0,
            roth_int_stock_add: 0.0,
            roth_int_bond_add: 0.0,
            roth_cash_add: 0.0,
            traditional_holdings: ShareValues::new(),
            traditional_us_stock_add: 0.0,
            traditional_us_bond_add: 0.0,
            traditional_int_stock_add: 0.0,
            traditional_int_bond_add: 0.0,
            traditional_cash_add: 0.0,
            use_brokerage_retirement: false,
            brokerage_holdings: ShareValues::new(),
            rebalance: VanguardRebalance::default(),
            vanguard_holdings: VanguardHoldings::default(),
            stock_quotes: ShareValues::new_quote(),
        }
    }
}

impl VaporeApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for VaporeApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("VAnguard POrtfolio REbalance");

            if ui.button("Open Vanguard File").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.vanguard_holdings = block_on(parse_csv_download(path)).unwrap();
                };
            };

            ui.horizontal(|ui| {
                ui.label("Brokerage account number:");
                egui::ComboBox::from_id_source("Brokerage")
                    .selected_text(self.brokerage_account_num.to_string())
                    .show_ui(ui, |ui| {
                        for acct_num in self.vanguard_holdings.accounts.keys() {
                            ui.selectable_value(&mut self.brokerage_account_num, *acct_num, acct_num.to_string());
                        };
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Traditional IRA account number:");
                egui::ComboBox::from_id_source("Traditional")
                    .selected_text(self.trad_account_num.to_string())
                    .show_ui(ui, |ui| {
                        for acct_num in self.vanguard_holdings.accounts.keys() {
                            ui.selectable_value(&mut self.trad_account_num, *acct_num, acct_num.to_string());
                        };
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Roth IRA account number:");
                egui::ComboBox::from_id_source("IRA")
                    .selected_text(self.roth_account_num.to_string())
                    .show_ui(ui, |ui| {
                        for acct_num in self.vanguard_holdings.accounts.keys() {
                            ui.selectable_value(&mut self.roth_account_num, *acct_num, acct_num.to_string());
                        };
                    });
            });

            if ui.button("Set account numbers").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.vanguard_holdings = block_on(parse_csv_download(path)).unwrap();
                    self.rebalance = to_buy(
                        self.percent_stock,
                        self.brokerage_cash_add,
                        self.brokerage_us_stock_add,
                        self.brokerage_int_stock_add,
                        self.brokerage_us_bond_add,
                        self.brokerage_int_bond_add,
                        self.retirement_year,
                        self.roth_holdings,
                        self.roth_us_stock_add,
                        self.roth_us_bond_add,
                        self.roth_int_stock_add,
                        self.roth_int_bond_add,
                        self.roth_cash_add,
                        self.traditional_holdings,
                        self.traditional_us_stock_add,
                        self.traditional_us_bond_add,
                        self.traditional_int_stock_add,
                        self.traditional_int_bond_add,
                        self.traditional_cash_add,
                        self.use_brokerage_retirement,
                        self.brokerage_holdings,
                        self.stock_quotes,
                        ).unwrap();
                };
            };

            ui.horizontal(|ui| {
                ui.vertical(|ui|{
                    ui.label("Symbol");
                    ui.label("Brokerage");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", symbol));
                    };
                    ui.label("Traditional IRA");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", symbol));
                    };
                    ui.label("Roth IRA");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", symbol));
                    };
                });
                ui.vertical(|ui|{
                    ui.label("Holdings");
                    ui.label("Brokerage");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.brokerage.current.stock_value(symbol)));
                    };
                    ui.label("Traditional IRA");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.traditional_ira.current.stock_value(symbol)));
                    }
                    ui.label("Roth IRA");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.roth_ira.current.stock_value(symbol)));
                    };
                });
                ui.vertical(|ui|{
                    ui.label("Target");
                    ui.label("Brokerage");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.brokerage.target.stock_value(symbol)));
                    };
                    ui.label("Traditional IRA");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.traditional_ira.target.stock_value(symbol)));
                    };
                    ui.label("Roth IRA");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.roth_ira.target.stock_value(symbol)));
                    };
                });
                ui.vertical(|ui|{
                    ui.label("Purchase");
                    ui.label("Brokerage");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.brokerage.sale_purchases_needed.stock_value(symbol)));
                    };
                    ui.label("Traditional IRA");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.traditional_ira.sale_purchases_needed.stock_value(symbol)));
                    };
                    ui.label("Roth IRA");
                    for symbol in StockSymbol::list() {
                        ui.label(format!("{:?}", self.rebalance.roth_ira.sale_purchases_needed.stock_value(symbol)));
                    };
                });
            });

            ui.add(egui::Slider::new(&mut self.brokerage_stock, 0..=100).text("Brokerage percentage stock"));

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
