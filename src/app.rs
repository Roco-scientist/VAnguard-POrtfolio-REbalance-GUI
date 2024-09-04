use crate::{
    calc,
    holdings::{parse_csv_download, ShareValues, StockSymbol, VanguardHoldings, VanguardRebalance},
};
#[cfg(not(target_arch = "wasm32"))]
use apca::{api::v2::account, ApiInfo, Client};
use chrono::{Datelike, Local};
#[cfg(not(target_arch = "wasm32"))]
use futures::executor::block_on;
use std::{
    collections::HashMap,
    future::Future,
    string::String,
    sync::{Arc, Mutex},
};

type ProfileName = String;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct VaporeApp {
    profile_name: ProfileName, // Holds the profile name that is used to retrieve birth year etc
    birth_year: HashMap<ProfileName, u32>, // Profile name: year
    retirement_year: HashMap<ProfileName, i32>, // Profile name: year
    brokerage_stock: HashMap<ProfileName, u32>, // Profile name: brokerage stock percent
    brokerage_account_num: HashMap<ProfileName, u32>, // Profile name: brokerage account number
    roth_account_num: HashMap<ProfileName, u32>, // Profile name: Roth account number
    trad_account_num: HashMap<ProfileName, u32>, // Profile name: Traditional IRA account number
    distribution_table: HashMap<u32, f32>, // Age: divider from IRS' distribution table
    #[serde(skip)]
    distribution_needed: String,
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    yahoo_updated: bool, // Year for which distributions are calculated
    #[serde(skip)]
    distribution_year: u32, // Year for which distributions are calculated
    #[serde(skip)]
    brokerage_cash_add: i32, // Amount of brokerage cahs added
    #[serde(skip)]
    brokerage_us_stock_add: f32, // US stock add to brokerage, e.g. Alpaca or manually added
    #[serde(skip)]
    brokerage_int_stock_add: f32, // Stock add unused at this time
    #[serde(skip)]
    brokerage_us_bond_add: f32, // Bond add unused at this time
    #[serde(skip)]
    brokerage_int_bond_add: f32, // Bond add unused at this time
    #[serde(skip)]
    roth_holdings: ShareValues, // Roth value holdings found from account number
    #[serde(skip)]
    roth_us_stock_add: f32, // Stock add unused at this time
    #[serde(skip)]
    roth_us_bond_add: f32, // Bond add unused at this time
    #[serde(skip)]
    roth_int_stock_add: f32, // Stock add unused at this time
    #[serde(skip)]
    roth_int_bond_add: f32, // Bond add unused at this time
    #[serde(skip)]
    roth_cash_add: i32, // Cash added or subtracted from Roth account
    #[serde(skip)]
    traditional_holdings: ShareValues, // Traditional stock values found with the account number
    #[serde(skip)]
    traditional_us_stock_add: f32, // Stock add unused at this time
    #[serde(skip)]
    traditional_us_bond_add: f32, // Bond add unused at this time
    #[serde(skip)]
    traditional_int_stock_add: f32, // Stock add unused at this time
    #[serde(skip)]
    traditional_int_bond_add: f32, // Bond add unused at this time
    #[serde(skip)]
    traditional_cash_add: i32, // Cash to add or subtract from the Traditional IRA
    use_brokerage_retirement: bool, // Whether to use the brokerage as the same allocation as retirement
    #[serde(skip)]
    brokerage_holdings: ShareValues, // Brokerage holdings found from the account number
    #[serde(skip)]
    rebalance: VanguardRebalance, // Targets and purchases/sales needed to rebalance
    #[serde(skip)]
    vanguard_holdings: Arc<Mutex<VanguardHoldings>>,
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
            distribution_needed: "Load distribution table for results. 1 year of VAPORE use needed.".to_string(),
            #[cfg(not(target_arch = "wasm32"))]
            yahoo_updated: false,
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
            vanguard_holdings: Arc::new(Mutex::new(VanguardHoldings::default())),
        }
    }
}

impl VaporeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Menu bar if not WASM website
            egui::menu::bar(ui, |ui| {
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
            ui.heading("VAnguard POrtfolio REbalance");

            // Button to import the Vanguard ofxdownload.csv file
            if ui.button("Open Vanguard File").clicked() {
                let file_future = rfd::AsyncFileDialog::new().pick_file();
                let vanguard_holdings = Arc::clone(&self.vanguard_holdings);
                // Function below used to import and be compatable with both OS and WASM
                execute(async move {
                    if let Some(file) = file_future.await {
                        *vanguard_holdings.lock().unwrap() =
                            parse_csv_download(String::from_utf8(file.read().await).unwrap())
                                .await
                                .unwrap();
                    }
                });
            };

            // If vanguard file is loaded, load the rest of the app
            if !self
                .vanguard_holdings
                .lock()
                .unwrap()
                .accounts_values
                .is_empty()
            {
                // Update with Yahoo quotes which is only possible when not WASM/website
                #[cfg(not(target_arch = "wasm32"))]
                ui.horizontal(|ui| {
                    if ui.button("Update with Yahoo stock quotes").clicked() {
                        self.vanguard_holdings
                            .lock()
                            .unwrap()
                            .update_with_yahoo_quotes()
                            .unwrap();
                        self.yahoo_updated = true;
                    };
                    if self.yahoo_updated {
                        ui.label("Updated");
                    }
                });

                // Profile creator with all values that are specific to each profile, such as account
                // numbers, birth year, and retirement year
                egui::CollapsingHeader::new("Profile").show(ui, |ui| {
                    // Selector for any previously made profiles
                    egui::ComboBox::from_id_source("Brokerage")
                        .selected_text(&self.profile_name)
                        .show_ui(ui, |ui| {
                            for profile in self.birth_year.keys() {
                                ui.selectable_value(
                                    &mut self.profile_name,
                                    profile.clone(),
                                    profile,
                                );
                            }
                        });

                    // Create or delete profile on a single horizontal frame
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.profile_name));
                        // Create new profile with the name from the text edit
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
                        // Delete profile with the name from the text edit.  Remove all profile name
                        // references in the profile HashMaps
                        if ui.button("Delete").clicked() {
                            self.birth_year.remove(&self.profile_name);
                            self.retirement_year.remove(&self.profile_name);
                            self.brokerage_stock.remove(&self.profile_name);
                            self.brokerage_account_num.remove(&self.profile_name);
                            self.roth_account_num.remove(&self.profile_name);
                            self.trad_account_num.remove(&self.profile_name);
                        }
                    });

                    // Birth year add
                    if let Some(birth_year) = self.birth_year.get_mut(&self.profile_name) {
                        ui.add(egui::Slider::new(&mut *birth_year, 1940..=2100).text("Birth year"));
                    }

                    // Retirement year add
                    if let Some(retirement_year) = self.retirement_year.get_mut(&self.profile_name)
                    {
                        ui.add(
                            egui::Slider::new(&mut *retirement_year, 2020..=2100)
                                .text("Retirement year"),
                        );
                    };

                    // If a profile has been created, allow selection of brokerage account numbers derived from
                    // the Vanguard download file
                    if let Some(profile_account_num) =
                        self.brokerage_account_num.get_mut(&self.profile_name)
                    {
                        ui.horizontal(|ui| {
                            ui.label("Brokerage account number:");
                            egui::ComboBox::from_id_source("Brokerage")
                                .selected_text(profile_account_num.to_string())
                                .show_ui(ui, |ui| {
                                    for acct_num in self
                                        .vanguard_holdings
                                        .lock()
                                        .unwrap()
                                        .accounts_values
                                        .keys()
                                    {
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

                    // If a profile has been created, allow selection of IRA account numbers derived from
                    // the Vanguard download file
                    if let Some(profile_account_num) =
                        self.trad_account_num.get_mut(&self.profile_name)
                    {
                        ui.horizontal(|ui| {
                            ui.label("Traditional IRA account number:");
                            egui::ComboBox::from_id_source("Traditional")
                                .selected_text(profile_account_num.to_string())
                                .show_ui(ui, |ui| {
                                    for acct_num in self
                                        .vanguard_holdings
                                        .lock()
                                        .unwrap()
                                        .accounts_values
                                        .keys()
                                    {
                                        ui.selectable_value(
                                            &mut *profile_account_num,
                                            *acct_num,
                                            acct_num.to_string(),
                                        );
                                    }
                                });
                        });
                    };

                    // If a profile has been created, allow selection of Roth IRA account numbers derived from
                    // the Vanguard download file
                    if let Some(profile_account_num) =
                        self.roth_account_num.get_mut(&self.profile_name)
                    {
                        ui.horizontal(|ui| {
                            ui.label("Roth IRA account number:");
                            egui::ComboBox::from_id_source("IRA")
                                .selected_text(profile_account_num.to_string())
                                .show_ui(ui, |ui| {
                                    for acct_num in self
                                        .vanguard_holdings
                                        .lock()
                                        .unwrap()
                                        .accounts_values
                                        .keys()
                                    {
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

                // If there is a profile created, set the brokerage hodlings to the account number
                // selected
                if let Some(brokerage_account_num) =
                    self.brokerage_account_num.get(&self.profile_name)
                {
                    self.brokerage_holdings = *self
                        .vanguard_holdings
                        .lock()
                        .unwrap()
                        .accounts_values
                        .get(brokerage_account_num)
                        .unwrap_or(&ShareValues::default());
                };

                // If there is a profile created, set the IRA hodlings to the account number
                // selected
                if let Some(trad_account_num) = self.trad_account_num.get(&self.profile_name) {
                    self.traditional_holdings = *self
                        .vanguard_holdings
                        .lock()
                        .unwrap()
                        .accounts_values
                        .get(trad_account_num)
                        .unwrap_or(&ShareValues::default());
                };

                // If there is a profile created, set the Roth IRA hodlings to the account number
                // selected
                if let Some(roth_account_num) = self.roth_account_num.get(&self.profile_name) {
                    self.roth_holdings = *self
                        .vanguard_holdings
                        .lock()
                        .unwrap()
                        .accounts_values
                        .get(roth_account_num)
                        .unwrap_or(&ShareValues::default());
                };

                // If brokerage percentage is not set by retirement ratios and is kept separate, create
                // a slider to input brokerage stock percent
                if !self.use_brokerage_retirement {
                    if let Some(brokerage_stock) = self.brokerage_stock.get_mut(&self.profile_name)
                    {
                        ui.add(
                            egui::Slider::new(&mut *brokerage_stock, 0..=100)
                                .text("Brokerage percentage stock"),
                        );
                    };
                };

                // Add outside US stock value to the brokerage account calculations
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.brokerage_us_stock_add, 0.0..=10000000.00)
                            .text("US stock value outside Vanguard"),
                    );
                    // If there is an Alpaca brokerage account, add that value
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Add Alpaca").clicked() {
                        let key_id =
                            std::env::var("APCA_API_KEY_ID").unwrap_or_else(|_| String::new());
                        let key =
                            std::env::var("APCA_API_SECRET_KEY").unwrap_or_else(|_| String::new());
                        if !key_id.is_empty() && !key.is_empty() {
                            let api_info =
                                ApiInfo::from_parts("https://api.alpaca.markets/", &key_id, &key)
                                    .unwrap();
                            let client = Client::new(api_info);
                            let alpaca_equity = block_on(client.issue::<account::Get>(&()))
                                .unwrap()
                                .equity
                                .to_f64()
                                .unwrap() as f32;
                            self.brokerage_us_stock_add += alpaca_equity;
                        } else {
                            ui.label("APCA_API_KEY_ID or APCA_API_SECRET_KEY missing");
                        };
                    };
                });

                // Cash to add or subtract from the brokerage account
                ui.add(
                    egui::Slider::new(&mut self.brokerage_cash_add, -100000..=100000)
                        .text("Brokerage cash add/remove"),
                );

                // Cash to add or subtract from the Roth IRA account
                ui.add(
                    egui::Slider::new(&mut self.roth_cash_add, -100000..=100000)
                        .text("Roth IRA cash add/remove"),
                );

                // Cash to add or subtract from the IRA account
                ui.add(
                    egui::Slider::new(&mut self.traditional_cash_add, -100000..=100000)
                        .text("Traditional IRA cash add/remove"),
                );

                // Distribution requirements after retirement age for the IRA.  Cannot be used by
                // WASM/website due to needing to get Yahoo quotes to determine the previous end of
                // year account value
                #[cfg(not(target_arch = "wasm32"))]
                egui::CollapsingHeader::new("Required distributions").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Load distribution table").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.distribution_table = calc::get_distribution_table(path).unwrap();
                                // If the profile's age is old enough, display the distribution needed
                                if let Some(birth_year) = self.birth_year.get(&self.profile_name) {
                                    if let Some(trad_account_num) =
                                        self.trad_account_num.get(&self.profile_name)
                                    {
                                        let age = self.distribution_year - birth_year;
                                        if age >= *self.distribution_table.keys().min().unwrap_or(&70) {
                                            let mut v_holdings = self.vanguard_holdings.lock().unwrap();
                                            if let Some(traditional_value) =
                                                block_on(v_holdings.eoy_value(
                                                    self.distribution_year,
                                                    *trad_account_num,
                                                ))
                                                .unwrap()
                                            {
                                                let minimum_distribution_div =
                                                    *self.distribution_table.get(&age).unwrap_or(&0.0);
                                                if minimum_distribution_div != 0.0 {
                                                    let minimum_distribution =
                                                        traditional_value / minimum_distribution_div;
                                                    let so_far = v_holdings.get_distributions(trad_account_num);
                                                    let left = (minimum_distribution - so_far).max(0.0);
                                                    self.distribution_needed = format!("Minimum distribution: ${:.2}  So far: ${:.2}  To go: ${:.2}", minimum_distribution, so_far, left);
                                                }
                                            } else {
                                                self.distribution_needed = "More transaction history needed".to_string();
                                            };
                                        }else{
                                            self.distribution_needed = "No distribution needed".to_string();
                                        };
                                    };
                                };
                            };
                        };

                        // Selection for current or previous year to calculated needed distributions
                        ui.label("Distribution year:");
                        ui.selectable_value(
                            &mut self.distribution_year,
                            Local::now().year() as u32 - 1,
                            (Local::now().year() as u32 - 1).to_string(),
                        );
                        ui.selectable_value(
                            &mut self.distribution_year,
                            Local::now().year() as u32,
                            Local::now().year().to_string(),
                        );

                    });
                    ui.label(self.distribution_needed.clone());
                });

                // Update the purchase/sales needed to rebalance the portfolio
                if let Some(brokerage_stock) = self.brokerage_stock.get(&self.profile_name) {
                    if let Some(retirement_year) = self.retirement_year.get(&self.profile_name) {
                        if ui.button("Update target holdings").clicked() {
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
                                self.vanguard_holdings.lock().unwrap().stock_quotes(),
                            )
                            .unwrap();
                        };
                    }
                }

                // Display the update hodlings within a drop menu
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

                // Display the updated target to rebalance the portfolio within a drop menu
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

                // Display the updated purchase/sales to rebalance the portfolio within a drop menu
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
            }

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
