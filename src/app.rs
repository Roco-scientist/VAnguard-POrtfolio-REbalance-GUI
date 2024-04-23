use crate::{
    calc,
    holdings::{parse_csv_download, ShareValues, StockSymbol, VanguardHoldings, VanguardRebalance},
};
use chrono::{Datelike, Local};
use futures::executor::block_on;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use apca::{api::v2::account, ApiInfo, Client};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct VaporeApp {
    profile_name: String,
    birth_year: HashMap<String, u32>,
    retirement_year: HashMap<String, i32>,
    brokerage_stock: HashMap<String, u32>,
    brokerage_account_num: HashMap<String, u32>,
    roth_account_num: HashMap<String, u32>,
    trad_account_num: HashMap<String, u32>,
    distribution_table: HashMap<u32, f32>,
    #[serde(skip)]
    distribution_year: u32,
    #[serde(skip)]
    brokerage_cash_add: i32,
    #[serde(skip)]
    brokerage_us_stock_add: f32,
    #[serde(skip)]
    brokerage_int_stock_add: f32,
    #[serde(skip)]
    brokerage_us_bond_add: f32,
    #[serde(skip)]
    brokerage_int_bond_add: f32,
    #[serde(skip)]
    roth_holdings: ShareValues,
    #[serde(skip)]
    roth_us_stock_add: f32,
    #[serde(skip)]
    roth_us_bond_add: f32,
    #[serde(skip)]
    roth_int_stock_add: f32,
    #[serde(skip)]
    roth_int_bond_add: f32,
    #[serde(skip)]
    roth_cash_add: i32,
    #[serde(skip)]
    traditional_holdings: ShareValues,
    #[serde(skip)]
    traditional_us_stock_add: f32,
    #[serde(skip)]
    traditional_us_bond_add: f32,
    #[serde(skip)]
    traditional_int_stock_add: f32,
    #[serde(skip)]
    traditional_int_bond_add: f32,
    #[serde(skip)]
    traditional_cash_add: i32,
    use_brokerage_retirement: bool,
    #[serde(skip)]
    brokerage_holdings: ShareValues,
    #[serde(skip)]
    rebalance: VanguardRebalance,
    #[serde(skip)]
    vanguard_holdings: VanguardHoldings,
    #[serde(skip)]
    stock_quotes: ShareValues,
}

impl Default for VaporeApp {
    fn default() -> Self {
        Self {
            profile_name: String::default(),
            birth_year: HashMap::new(),
            retirement_year: HashMap::new(),
            brokerage_stock: HashMap::new(),
            brokerage_account_num: HashMap::new(),
            roth_account_num: HashMap::new(),
            trad_account_num: HashMap::new(),
            distribution_table: HashMap::new(),
            distribution_year: Local::now().year() as u32,
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

            egui::CollapsingHeader::new("Profile").show(ui, |ui| {
                egui::ComboBox::from_id_source("Brokerage")
                    .selected_text(&self.profile_name)
                    .show_ui(ui, |ui| {
                        for profile in self.birth_year.keys() {
                            ui.selectable_value(&mut self.profile_name, profile.clone(), profile);
                        }
                    });
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.profile_name));
                    if ui.button("Create").clicked() {
                        if !self.birth_year.contains_key(&self.profile_name) {
                            self.birth_year.insert(self.profile_name.clone(), 1980);
                        };
                        if !self.retirement_year.contains_key(&self.profile_name) {
                            self.retirement_year.insert(self.profile_name.clone(), 2050);
                        };
                        if !self.brokerage_stock.contains_key(&self.profile_name) {
                            self.brokerage_stock.insert(self.profile_name.clone(), 65);
                        };
                        if !self.brokerage_account_num.contains_key(&self.profile_name) {
                            self.brokerage_account_num
                                .insert(self.profile_name.clone(), 0);
                        };
                        if !self.roth_account_num.contains_key(&self.profile_name) {
                            self.roth_account_num.insert(self.profile_name.clone(), 0);
                        };
                        if !self.trad_account_num.contains_key(&self.profile_name) {
                            self.trad_account_num.insert(self.profile_name.clone(), 0);
                        };
                    };
                    if ui.button("Delete").clicked() {
                        self.birth_year.remove(&self.profile_name);
                        self.retirement_year.remove(&self.profile_name);
                        self.brokerage_stock.remove(&self.profile_name);
                        self.brokerage_account_num.remove(&self.profile_name);
                        self.roth_account_num.remove(&self.profile_name);
                        self.trad_account_num.remove(&self.profile_name);
                    }
                });

                if let Some(birth_year) = self.birth_year.get_mut(&self.profile_name) {
                    ui.add(egui::Slider::new(&mut *birth_year, 1940..=2100).text("Birth year"));
                }

                if let Some(retirement_year) = self.retirement_year.get_mut(&self.profile_name) {
                    ui.add(
                        egui::Slider::new(&mut *retirement_year, 2020..=2100)
                            .text("Retirement year"),
                    );
                };

                if let Some(profile_account_num) =
                    self.brokerage_account_num.get_mut(&self.profile_name)
                {
                    ui.horizontal(|ui| {
                        ui.label("Brokerage account number:");
                        egui::ComboBox::from_id_source("Brokerage")
                            .selected_text(profile_account_num.to_string())
                            .show_ui(ui, |ui| {
                                for acct_num in self.vanguard_holdings.accounts.keys() {
                                    ui.selectable_value(
                                        &mut *profile_account_num,
                                        *acct_num,
                                        acct_num.to_string(),
                                    );
                                }
                            });
                        ui.checkbox(&mut self.use_brokerage_retirement, "Retirement");
                    });
                };
                if let Some(profile_account_num) = self.trad_account_num.get_mut(&self.profile_name)
                {
                    ui.horizontal(|ui| {
                        ui.label("Traditional IRA account number:");
                        egui::ComboBox::from_id_source("Traditional")
                            .selected_text(profile_account_num.to_string())
                            .show_ui(ui, |ui| {
                                for acct_num in self.vanguard_holdings.accounts.keys() {
                                    ui.selectable_value(
                                        &mut *profile_account_num,
                                        *acct_num,
                                        acct_num.to_string(),
                                    );
                                }
                            });
                    });
                };
                if let Some(profile_account_num) = self.roth_account_num.get_mut(&self.profile_name)
                {
                    ui.horizontal(|ui| {
                        ui.label("Roth IRA account number:");
                        egui::ComboBox::from_id_source("IRA")
                            .selected_text(profile_account_num.to_string())
                            .show_ui(ui, |ui| {
                                for acct_num in self.vanguard_holdings.accounts.keys() {
                                    ui.selectable_value(
                                        &mut *profile_account_num,
                                        *acct_num,
                                        acct_num.to_string(),
                                    );
                                }
                            });
                    });
                };
            });

            if let Some(brokerage_account_num) = self.brokerage_account_num.get(&self.profile_name)
            {
                self.brokerage_holdings = self
                    .vanguard_holdings
                    .accounts
                    .get(brokerage_account_num)
                    .unwrap_or(&ShareValues::default())
                    .clone();
            };
            if let Some(trad_account_num) = self.trad_account_num.get(&self.profile_name) {
                self.traditional_holdings = self
                    .vanguard_holdings
                    .accounts
                    .get(trad_account_num)
                    .unwrap_or(&ShareValues::default())
                    .clone();
            };
            if let Some(roth_account_num) = self.roth_account_num.get(&self.profile_name) {
                self.roth_holdings = self
                    .vanguard_holdings
                    .accounts
                    .get(roth_account_num)
                    .unwrap_or(&ShareValues::default())
                    .clone();
            };

            if !self.use_brokerage_retirement {
                if let Some(brokerage_stock) = self.brokerage_stock.get_mut(&self.profile_name) {
                    ui.add(
                        egui::Slider::new(&mut *brokerage_stock, 0..=100)
                            .text("Brokerage percentage stock"),
                    );
                };
            };

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.brokerage_us_stock_add, 0.0..=10000000.00)
                        .text("US stock value outside Vanguard"),
                );
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Add Alpaca").clicked() {
                    let key_id = std::env::var("APCA_API_KEY_ID").unwrap_or_else(|_| String::new());
                    let key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_else(|_| String::new());
                    if !key_id.is_empty() && !key.is_empty() {
                        let api_info = ApiInfo::from_parts("https://api.alpaca.markets/", &key_id, &key)
                            .unwrap();
                        let client = Client::new(api_info);
                        let alpaca_equity = block_on(client
                            .issue::<account::Get>(&())).unwrap()
                            .equity
                            .to_f64()
                            .unwrap() as f32;
                        self.brokerage_us_stock_add += alpaca_equity;
                    }else{
                        ui.label("APCA_API_KEY_ID or APCA_API_SECRET_KEY missing");
                    };
                };
            });

            ui.add(
                egui::Slider::new(&mut self.brokerage_cash_add, -100000..=100000)
                    .text("Brokerage cash add/remove"),
            );

            ui.add(
                egui::Slider::new(&mut self.roth_cash_add, -100000..=100000)
                    .text("Roth IRA cash add/remove"),
            );

            ui.add(
                egui::Slider::new(&mut self.traditional_cash_add, -100000..=100000)
                    .text("Traditional IRA cash add/remove"),
            );

            ui.horizontal(|ui| {
                if ui.button("Load distribution table").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.distribution_table = calc::get_distribution_table(path).unwrap();
                    };
                };

                let current_year = Local::now().year() as u32;
                let last_year = current_year - 1;
                ui.label("Distribution year:");
                ui.selectable_value(
                    &mut self.distribution_year,
                    last_year,
                    last_year.to_string(),
                );
                ui.selectable_value(
                    &mut self.distribution_year,
                    current_year,
                    current_year.to_string(),
                );
                if let Some(birth_year) = self.birth_year.get(&self.profile_name) {
                    if let Some(trad_account_num) = self.trad_account_num.get(&self.profile_name) {
                        let age = self.distribution_year - birth_year;
                        if age > self.distribution_table.keys().min().unwrap_or(&70).clone() {
                            if let Some(traditional_value) = block_on(
                                self.vanguard_holdings
                                    .eoy_value(self.distribution_year, trad_account_num.clone()),
                            )
                            .unwrap()
                            {
                                let minimum_distribution_div =
                                    self.distribution_table.get(&age).unwrap_or(&0.0).clone();
                                if minimum_distribution_div != 0.0 {
                                    let minimum_distribution =
                                        traditional_value / minimum_distribution_div;
                                    let so_far =
                                        self.vanguard_holdings.distributions(trad_account_num);
                                    let left = (minimum_distribution - so_far).max(0.0);
                                    ui.label(format!(
                                        "Minimum distribution: {:.2}",
                                        minimum_distribution
                                    ));
                                    ui.label(format!("So far: {:.2}", so_far));
                                    ui.label(format!("To go: {:.2}", left));
                                }
                            } else {
                                ui.label("More transaction history needed");
                            };
                        };
                    };
                };
            });

            if let Some(brokerage_stock) = self.brokerage_stock.get(&self.profile_name) {
                if let Some(retirement_year) = self.retirement_year.get(&self.profile_name) {
                    if ui.button("Update").clicked() {
                        block_on(self.stock_quotes.add_missing_quotes()).unwrap();
                        self.rebalance = calc::to_buy(
                            *brokerage_stock as f32,
                            self.brokerage_cash_add as f32,
                            self.brokerage_us_stock_add,
                            self.brokerage_int_stock_add,
                            self.brokerage_us_bond_add,
                            self.brokerage_int_bond_add,
                            *retirement_year,
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
                }
            }

            egui::CollapsingHeader::new("Holdings").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Symbol");
                        for symbol in StockSymbol::list() {
                            ui.label(format!("{:?}", symbol));
                        }
                        ui.label("Other");
                    });
                    ui.vertical(|ui| {
                        ui.label("Brokerage");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.brokerage.current.stock_value(symbol)
                            ));
                        }
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .brokerage
                                .current
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
                    });
                    ui.vertical(|ui| {
                        ui.label("Traditional IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.traditional_ira.current.stock_value(symbol)
                            ));
                        }
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .traditional_ira
                                .current
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
                    });
                    ui.vertical(|ui| {
                        ui.label("Roth IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.roth_ira.current.stock_value(symbol)
                            ));
                        }
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .roth_ira
                                .current
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
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
                        ui.label("Other");
                    });
                    ui.vertical(|ui| {
                        ui.label("Brokerage");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.brokerage.target.stock_value(symbol)
                            ));
                        }
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .brokerage
                                .target
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
                    });
                    ui.vertical(|ui| {
                        ui.label("Traditional IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.traditional_ira.target.stock_value(symbol)
                            ));
                        }
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .traditional_ira
                                .target
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
                    });
                    ui.vertical(|ui| {
                        ui.label("Roth IRA");
                        for symbol in StockSymbol::list() {
                            ui.label(format!(
                                "{:.1}",
                                self.rebalance.roth_ira.target.stock_value(symbol)
                            ));
                        }
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .roth_ira
                                .target
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
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
                        ui.label("Other");
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
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .brokerage
                                .sale_purchases_needed
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
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
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .traditional_ira
                                .sale_purchases_needed
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
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
                        ui.label(format!(
                            "{:.1}",
                            self.rebalance
                                .roth_ira
                                .sale_purchases_needed
                                .stock_value(StockSymbol::Other(String::default()))
                        ));
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
