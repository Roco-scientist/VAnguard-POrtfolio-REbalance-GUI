use crate::{
    calc,
    holdings::{parse_csv_download, ShareValues, StockSymbol, VanguardHoldings, VanguardRebalance},
};
use futures::executor::block_on;
use std::collections::HashMap;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct VaporeApp {
    // Example stuff:
    birth_year: u32,
    retirement_year: i32,
    brokerage_stock: u32,
    brokerage_account_num: u32,
    roth_account_num: u32,
    trad_account_num: u32,
    distribution_table: HashMap<u32, f32>,
    distribution_year: u32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    brokerage_cash_add: i32,
    brokerage_us_stock_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    brokerage_int_stock_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    brokerage_us_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    brokerage_int_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    roth_holdings: ShareValues,
    #[serde(skip)] // This how you opt-out of serialization of a field
    roth_us_stock_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    roth_us_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    roth_int_stock_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    roth_int_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    roth_cash_add: i32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    traditional_holdings: ShareValues,
    #[serde(skip)] // This how you opt-out of serialization of a field
    traditional_us_stock_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    traditional_us_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    traditional_int_stock_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    traditional_int_bond_add: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    traditional_cash_add: i32,
    use_brokerage_retirement: bool,
    #[serde(skip)] // This how you opt-out of serialization of a field
    brokerage_holdings: ShareValues,
    #[serde(skip)] // This how you opt-out of serialization of a field
    rebalance: VanguardRebalance,
    #[serde(skip)] // This how you opt-out of serialization of a field
    vanguard_holdings: VanguardHoldings,
    #[serde(skip)] // This how you opt-out of serialization of a field
    stock_quotes: ShareValues,
}

impl Default for VaporeApp {
    fn default() -> Self {
        Self {
            birth_year: 1980,
            retirement_year: 2025,
            brokerage_stock: 65,
            brokerage_account_num: 0,
            roth_account_num: 0,
            trad_account_num: 0,
            distribution_table: HashMap::new(),
            distribution_year: 2025,
            brokerage_cash_add: 0,
            brokerage_us_stock_add: 0.0,
            brokerage_int_stock_add: 0.0,
            brokerage_us_bond_add: 0.0,
            brokerage_int_bond_add: 0.0,
            roth_holdings: ShareValues::new(),
            roth_us_stock_add: 0.0,
            roth_us_bond_add: 0.0,
            roth_int_stock_add: 0.0,
            roth_int_bond_add: 0.0,
            roth_cash_add: 0,
            traditional_holdings: ShareValues::new(),
            traditional_us_stock_add: 0.0,
            traditional_us_bond_add: 0.0,
            traditional_int_stock_add: 0.0,
            traditional_int_bond_add: 0.0,
            traditional_cash_add: 0,
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
                            ui.selectable_value(
                                &mut self.brokerage_account_num,
                                *acct_num,
                                acct_num.to_string(),
                            );
                        }
                    });
                ui.checkbox(&mut self.use_brokerage_retirement, "Retirement");
            });
            ui.horizontal(|ui| {
                ui.label("Traditional IRA account number:");
                egui::ComboBox::from_id_source("Traditional")
                    .selected_text(self.trad_account_num.to_string())
                    .show_ui(ui, |ui| {
                        for acct_num in self.vanguard_holdings.accounts.keys() {
                            ui.selectable_value(
                                &mut self.trad_account_num,
                                *acct_num,
                                acct_num.to_string(),
                            );
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Roth IRA account number:");
                egui::ComboBox::from_id_source("IRA")
                    .selected_text(self.roth_account_num.to_string())
                    .show_ui(ui, |ui| {
                        for acct_num in self.vanguard_holdings.accounts.keys() {
                            ui.selectable_value(
                                &mut self.roth_account_num,
                                *acct_num,
                                acct_num.to_string(),
                            );
                        }
                    });
            });

            self.brokerage_holdings = self
                .vanguard_holdings
                .accounts
                .get(&self.brokerage_account_num)
                .unwrap_or(&ShareValues::default())
                .clone();
            self.traditional_holdings = self
                .vanguard_holdings
                .accounts
                .get(&self.trad_account_num)
                .unwrap_or(&ShareValues::default())
                .clone();
            self.roth_holdings = self
                .vanguard_holdings
                .accounts
                .get(&self.roth_account_num)
                .unwrap_or(&ShareValues::default())
                .clone();

            if !self.use_brokerage_retirement {
                ui.add(
                    egui::Slider::new(&mut self.brokerage_stock, 0..=100)
                        .text("Brokerage percentage stock"),
                );
            }

            ui.add(
                egui::Slider::new(&mut self.retirement_year, 2020..=2100)
                    .text("Retirement year"),
            );

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.birth_year, 1940..=2100)
                        .text("Birth year"),
                );

                if ui.button("Load distribution table").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.distribution_table = calc::get_distribution_table(path).unwrap();
                    };
                };

                ui.add(
                    egui::Slider::new(&mut self.distribution_year, 2020..=2100)
                        .text("Distribution year"),
                );
                let age = self.distribution_year - self.birth_year;
                if age > 72 {
                    if let Some(traditional_value) = block_on(self.vanguard_holdings.eoy_value(self.distribution_year, self.trad_account_num)).unwrap() {
                        let minimum_distribution_div = self.distribution_table.get(&age).unwrap_or(&0.0).clone();
                        if minimum_distribution_div != 0.0 {
                            let minimum_distribution = traditional_value / minimum_distribution_div;
                            let so_far = self.vanguard_holdings.distributions(&self.trad_account_num);
                            let left = (minimum_distribution - so_far).max(0.0);
                            ui.label(format!("Minimum distribution: {:.2}", minimum_distribution));
                            ui.label(format!("So far: {:.2}", so_far));
                            ui.label(format!("To go: {:.2}", left));
                        }
                    }
                }
            });

            ui.add(
                egui::Slider::new(&mut self.brokerage_cash_add, -100000..=100000)
                    .text("Brokerage cash add/remove"),
            );

            ui.add(
                egui::Slider::new(&mut self.traditional_cash_add, -100000..=100000)
                    .text("Traditional IRA cash add/remove"),
            );

            ui.add(
                egui::Slider::new(&mut self.roth_cash_add, -100000..=100000)
                    .text("Roth IRA cash add/remove"),
            );

            ui.add(
                egui::Slider::new(&mut self.brokerage_us_stock_add, 0.0..=10000000.00)
                    .text("US stock value outside Vanguard"),
            );

            if ui.button("Update").clicked() {
                block_on(self.stock_quotes.add_missing_quotes()).unwrap();
                self.rebalance = calc::to_buy(
                    self.brokerage_stock as f32,
                    self.brokerage_cash_add as f32,
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
                    self.roth_cash_add as f32,
                    self.traditional_holdings,
                    self.traditional_us_stock_add,
                    self.traditional_us_bond_add,
                    self.traditional_int_stock_add,
                    self.traditional_int_bond_add,
                    self.traditional_cash_add as f32,
                    self.use_brokerage_retirement,
                    self.brokerage_holdings,
                    self.stock_quotes,
                )
                .unwrap();
            };

            egui::CollapsingHeader::new("Holdings").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Symbol");
                        for symbol in StockSymbol::list() {
                            ui.label(format!("{:?}", symbol));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Brokerage");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.brokerage.current.stock_value(symbol)
                            ));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Traditional IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.traditional_ira.current.stock_value(symbol)
                            ));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Roth IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.roth_ira.current.stock_value(symbol)
                            ));
                        }
                    });
                });
            });
            egui::CollapsingHeader::new("Target").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Symbol");
                        for symbol in StockSymbol::list() {
                            ui.label(format!("{:?}", symbol));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Brokerage");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.brokerage.target.stock_value(symbol)
                            ));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Traditional IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.traditional_ira.target.stock_value(symbol)
                            ));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Roth IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.roth_ira.target.stock_value(symbol)
                            ));
                        }
                    });
                });
            });
            egui::CollapsingHeader::new("Purchase").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Symbol");
                        for symbol in StockSymbol::list() {
                            ui.label(format!("{:?}", symbol));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Brokerage");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance
                                    .brokerage
                                    .sale_purchases_needed
                                    .stock_value(symbol)
                            ));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Traditional IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance
                                    .traditional_ira
                                    .sale_purchases_needed
                                    .stock_value(symbol)
                            ));
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Roth IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance
                                    .roth_ira
                                    .sale_purchases_needed
                                    .stock_value(symbol)
                            ));
                        }
                    });
                });
            });
            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
