use crate::asset::SubAllocations;
use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate};
use std::{
    collections::HashMap,
    fmt,
    ops::{Add, Div, Mul, Sub},
    vec::Vec,
};
#[cfg(not(target_arch = "wasm32"))]
use time::{macros::format_description, OffsetDateTime};
#[cfg(not(target_arch = "wasm32"))]
use yahoo_finance_api as yahoo;

// STOCK_DESCRIPTION holds the descriptions for the stock symbols which is used to print and
// display
lazy_static! {
    static ref STOCK_DESCRIPTION: HashMap<StockSymbol, &'static str> = {
        let mut m = HashMap::new();
        m.insert(StockSymbol::VV, "US large cap");
        m.insert(StockSymbol::VO, "US mid cap");
        m.insert(StockSymbol::VB, "US small cap");
        m.insert(StockSymbol::VTC, "US total corporate bond");
        m.insert(StockSymbol::BND, "US total bond");
        m.insert(StockSymbol::VXUS, "Total international stock");
        m.insert(StockSymbol::VWO, "Emerging markets stock");
        m.insert(StockSymbol::BNDX, "Total international bond");
        m.insert(StockSymbol::VTIP, "Inflation protected securities");
        m
    };
}

/// StockSymbol is an enum which holds all stock symbols which are supported.  Empty is used to
/// initiated structs which use this enum.  Other<String> is a holder of any stock that is not
/// supported, where the String is the stock symbol.
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum StockSymbol {
    VXUS,
    BNDX,
    VTIP,
    BND,
    VWO,
    VO,
    VB,
    VTC,
    VV,
    VMFXX,
    Empty,
    Other(String),
}

impl StockSymbol {
    /// new creates a new StockSymbol enum based on the string value.
    ///
    ///  # Example
    ///
    ///  ```
    ///  use vapore::holdings::StockSymbol;
    ///
    ///  let bnd = StockSymbol::new("BND");
    ///  assert_eq!(bnd, StockSymbol::BND);
    ///  ```
    pub fn new(symbol: &str) -> Self {
        match symbol {
            "VXUS" => StockSymbol::VXUS,
            "BNDX" => StockSymbol::BNDX,
            "VTIP" => StockSymbol::VTIP,
            "BND" => StockSymbol::BND,
            "VWO" => StockSymbol::VWO,
            "VO" => StockSymbol::VO,
            "VB" => StockSymbol::VB,
            "VTC" => StockSymbol::VTC,
            "VV" => StockSymbol::VV,
            "VMFXX" => StockSymbol::VMFXX,
            "" => StockSymbol::Empty,
            _ => StockSymbol::Other(symbol.to_string()),
        }
    }

    /// description returns a string of the StockSymbol description.  If the stock is not
    /// supported, a "No description" String is returned.
    ///
    /// # Example
    ///
    /// ```
    ///  use vapore::holdings::StockSymbol;
    ///
    ///  let bnd = StockSymbol::new("BND");
    ///  let bnd_description = bnd.description();
    ///  assert_eq!(bnd_description, "BND: US total bond");
    ///
    /// ```
    pub fn description(&self) -> String {
        let description_option = STOCK_DESCRIPTION.get(self);
        if let Some(description) = description_option {
            format!("{:?}: {}", self, description)
        } else {
            format!("No description for {:?}", self)
        }
    }

    pub fn list() -> [StockSymbol; 9] {
        [
            StockSymbol::VV,
            StockSymbol::VO,
            StockSymbol::VB,
            StockSymbol::VTC,
            StockSymbol::BND,
            StockSymbol::VXUS,
            StockSymbol::VWO,
            StockSymbol::BNDX,
            StockSymbol::VTIP,
        ]
    }
}

/// all_stock_descriptions returns a String containing the description of all stocks which are
/// supported with each separated by a new line.  This is used to display on screen or write to
/// file all of the descriptions.
///
/// # Example
///
/// ```
/// use vapore::holdings;
///
/// let descriptions = holdings::all_stock_descriptions();
/// println!("{}", descriptions);
///
/// ```
pub fn all_stock_descriptions() -> String {
    let mut descriptions = String::new();
    for symbol in [
        StockSymbol::VV,
        StockSymbol::VO,
        StockSymbol::VB,
        StockSymbol::VTC,
        StockSymbol::BND,
        StockSymbol::VXUS,
        StockSymbol::VWO,
        StockSymbol::BNDX,
        StockSymbol::VTIP,
    ] {
        descriptions.push_str(&symbol.description());
        descriptions.push('\n')
    }
    descriptions.pop();
    descriptions
}

#[derive(Clone)]
pub struct StockInfo {
    pub account_number: u32,
    pub symbol: StockSymbol,
    pub share_price: f32,
    pub shares: f32,
    pub total_value: f32,
    account_added: bool,
    symbol_added: bool,
    share_price_added: bool,
    shares_added: bool,
    total_value_added: bool,
}

impl StockInfo {
    /// new initializes a new StockInfo struct.  Account number, symbol, share price etc. can then
    /// be added with the other methods.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// ```
    pub fn new() -> Self {
        StockInfo {
            account_number: 0,
            symbol: StockSymbol::Empty,
            share_price: 0.0,
            shares: 0.0,
            total_value: 0.0,
            account_added: false,
            symbol_added: false,
            share_price_added: false,
            shares_added: false,
            total_value_added: false,
        }
    }

    /// add_account adds the vanguard account number to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    ///
    /// assert_eq!(new_stock.account_number, 123456789);
    /// ```
    pub fn add_account(&mut self, account_number: u32) {
        self.account_number = account_number;
        self.account_added = true;
    }

    /// add_symbol adds the stock symbol to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    ///
    /// assert_eq!(new_stock.symbol, holdings::StockSymbol::BND);
    /// ```
    pub fn add_symbol(&mut self, symbol: StockSymbol) {
        self.symbol = symbol;
        self.symbol_added = true;
    }

    /// add_share_price adds the stock quote price to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    ///
    /// assert_eq!(new_stock.share_price, 234.50);
    /// ```
    pub fn add_share_price(&mut self, share_price: f32) {
        self.share_price = share_price;
        self.share_price_added = true;
    }

    /// add_share adds the stock total shares to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    /// new_stock.add_shares(10.0)
    ///
    /// assert_eq!(new_stock.shares, 10.0);
    /// ```
    pub fn add_shares(&mut self, share_num: f32) {
        self.shares = share_num;
        self.shares_added = true;
    }

    /// add_total_value adds the account total value of the stock to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    /// new_stock.add_total_value(5000.00);
    ///
    /// assert_eq!(new_stock.total_value, 5000.00);
    /// ```
    pub fn add_total_value(&mut self, total_value: f32) {
        self.total_value = total_value;
        self.total_value_added = true;
    }

    /// finished returns a bool of whether or not all struct values have been added.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    /// new_stock.add_total_value(5000.00);
    /// new_stock.add_shares(10.0);
    ///
    /// assert!(new_stock.finished());
    ///
    /// let empty_stock = holdings::StockInfo::new();
    /// assert!(!empty_stock.finished())
    /// ```
    pub fn finished(&self) -> bool {
        [
            self.account_added,
            self.symbol_added,
            self.share_price_added,
            self.shares_added,
            self.total_value_added,
        ]
        .iter()
        .all(|value| *value)
    }
}

impl Default for StockInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_yahoo_quote(stock_symbol: StockSymbol) -> Result<f32> {
    let stock_str = match stock_symbol {
        StockSymbol::VO => "VO",
        StockSymbol::VB => "VB",
        StockSymbol::VV => "VV",
        StockSymbol::BND => "BND",
        StockSymbol::VWO => "VWO",
        StockSymbol::VTC => "VTC",
        StockSymbol::VXUS => "VXUS",
        StockSymbol::BNDX => "BNDX",
        StockSymbol::VTIP => "VTIP",
        _ => "none",
    };
    if stock_str == "none" {
        Ok(1.0)
    } else {
        let provider = yahoo::YahooConnector::new();
        let response_err = provider
            .get_latest_quotes(stock_str, "1m")
            .await
            .with_context(|| format!("Latest quote error for: {}", stock_str));
        // If the market is closed, an error occurs.  If so, get quote history then the last quote
        if let Ok(response) = response_err {
            Ok(response.last_quote()?.close as f32)
        } else {
            let today = OffsetDateTime::now_utc();
            let week_ago = today - time::Duration::days(7);
            let response = provider
                .get_quote_history(stock_str, week_ago, today)
                .await
                .with_context(|| {
                    format!("Both attempts at quote retrieval failed for: {}", stock_str)
                })?;
            Ok(response.last_quote()?.close as f32)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_yahoo_eoy_quote(stock_symbol: StockSymbol, year: u32) -> Result<f32> {
    let stock_str = match stock_symbol {
        StockSymbol::VO => "VO",
        StockSymbol::VB => "VB",
        StockSymbol::VV => "VV",
        StockSymbol::BND => "BND",
        StockSymbol::VWO => "VWO",
        StockSymbol::VTC => "VTC",
        StockSymbol::VXUS => "VXUS",
        StockSymbol::BNDX => "BNDX",
        StockSymbol::VTIP => "VTIP",
        _ => "none",
    };
    if stock_str == "none" {
        Ok(1.0)
    } else {
        let provider = yahoo::YahooConnector::new();
        let format = format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]"
        );
        let start = OffsetDateTime::parse(&format!("{}-12-25 00:00:01 -05", year), format)?;
        let stop = OffsetDateTime::parse(&format!("{}-12-31 23:59:59 -05", year), format)?;
        let response = provider
            .get_quote_history(stock_str, start, stop)
            .await
            .with_context(|| format!("Quote history error for: {}", stock_str))?;
        Ok(response.quotes()?.last().unwrap().close as f32)
    }
}

/// AddType is an enum used to distinguish between when a stock quote or an account holdings is
/// wanted for input into a ShareValues struct.
pub enum AddType {
    StockPrice,
    HoldingValue,
    HoldingShares,
}

/// ShareValues holds the values for the supported ETF stocks.  The value can represent price,
/// holding value, stock quantity etc.
#[derive(Clone, PartialEq, Debug, Copy)]
pub struct ShareValues {
    vxus: f32,
    bndx: f32,
    bnd: f32,
    vwo: f32,
    vo: f32,
    vb: f32,
    vtc: f32,
    vv: f32,
    vtip: f32,
    vmfxx: f32,
    other: f32,
    outside_bond: f32,
    outside_stock: f32,
}

impl ShareValues {
    /// new creates a new ShareValues struct where all values are set to 0.  This is used within
    /// vapore to create a new struct for account holdings, etc.
    ///
    /// # Example
    /// ```
    /// use vapore::holdings;
    ///
    /// let new_values = holdings::ShareValues::new();
    /// ```
    pub fn new() -> Self {
        ShareValues {
            vxus: 0.0,
            bndx: 0.0,
            vtip: 0.0,
            bnd: 0.0,
            vwo: 0.0,
            vo: 0.0,
            vb: 0.0,
            vtc: 0.0,
            vv: 0.0,
            other: 0.0,
            vmfxx: 0.0,
            outside_bond: 0.0,
            outside_stock: 0.0,
        }
    }

    pub fn value_added(&self, default_value: f32) -> bool {
        [
            self.vxus,
            self.bndx,
            self.vtip,
            self.bnd,
            self.vwo,
            self.vo,
            self.vb,
            self.vtc,
            self.vv,
            self.vmfxx,
            self.outside_bond,
            self.outside_stock,
        ]
        .iter()
        .any(|val| val != &default_value)
    }
    /// new_quote creates a new ShareValues struct where all values are set to 1.  This is used for
    /// creating a new struct for stock quotes.  This way if any quotes are missing, they are
    /// automatically set to 1 to prevent any 0 division errors.  This also has the effect of
    /// outputting the dollar amount when target value is divided by quote price.  This division
    /// occurs to determine number of stocks to purchase/sell.
    ///
    /// # Example
    /// ```
    /// use vapore::holdings;
    ///
    /// let new_quotes = holdings::ShareValues::new_quote();
    /// ```
    pub fn new_quote() -> Self {
        ShareValues {
            vxus: 1.0,
            bndx: 1.0,
            vtip: 1.0,
            bnd: 1.0,
            vwo: 1.0,
            vo: 1.0,
            vb: 1.0,
            vtc: 1.0,
            vv: 1.0,
            vmfxx: 1.0,
            other: 1.0,
            outside_bond: 1.0,
            outside_stock: 1.0,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn add_missing_quotes(&mut self) -> Result<()> {
        for stock_symbol in [
            StockSymbol::VV,
            StockSymbol::VO,
            StockSymbol::VB,
            StockSymbol::VTC,
            StockSymbol::BND,
            StockSymbol::VXUS,
            StockSymbol::VWO,
            StockSymbol::BNDX,
            StockSymbol::VTIP,
        ] {
            if self.stock_value(stock_symbol.clone()) == 1.0 {
                let new_quote = get_yahoo_quote(stock_symbol.clone()).await?;
                self.add_stock_value(stock_symbol, new_quote);
            }
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn add_missing_eoy_quotes(&mut self, year: u32) -> Result<()> {
        for stock_symbol in [
            StockSymbol::VV,
            StockSymbol::VO,
            StockSymbol::VB,
            StockSymbol::VTC,
            StockSymbol::BND,
            StockSymbol::VXUS,
            StockSymbol::VWO,
            StockSymbol::BNDX,
            StockSymbol::VTIP,
        ] {
            if self.stock_value(stock_symbol.clone()) == 1.0 {
                let new_quote = get_yahoo_eoy_quote(stock_symbol.clone(), year).await?;
                self.add_stock_value(stock_symbol, new_quote);
            }
        }
        Ok(())
    }

    /// new_target creates a new target ShareValues struct which determines what to what values to
    /// rebalance to vanguard portfolio.
    ///
    /// # Panic
    ///
    /// Panics when the percentages and fractions do not add up to 1 when they are added together.
    /// This is necessary to make sure everything adds up to 100% of the total portfolio.  Adding
    /// up to less or more than 100% can happen when the const values determining balance distribution
    /// are changed without changing other values to make sure everything adds up.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::{asset, holdings};
    ///
    /// let sub_allocations = asset::SubAllocations::new().unwrap();
    ///
    /// let brokerage_target = holdings::ShareValues::new_target(sub_allocations, 10000.0, 0.0, 0.0, 0.0, 0.0);
    /// ```
    pub fn new_target(
        sub_allocations: SubAllocations,
        total_vanguard_value: f32,
        other_us_stock_value: f32,
        other_us_bond_value: f32,
        other_int_stock_value: f32,
        other_int_bond_value: f32,
    ) -> Self {
        // get total value
        let total_value = total_vanguard_value
            + other_us_stock_value
            + other_us_bond_value
            + other_int_bond_value
            + other_int_stock_value;

        // Calculate values for each stock
        let vxus_value = (total_value * sub_allocations.int_tot_stock / 100.0)
            - (other_int_stock_value * 2.0 / 3.0);
        let bndx_value = (total_value * sub_allocations.int_bond / 100.0) - other_int_bond_value;
        let bnd_value =
            (total_value * sub_allocations.us_tot_bond / 100.0) - (other_us_bond_value / 2.0);
        let vwo_value = (total_value * sub_allocations.int_emerging_stock / 100.0)
            - (other_int_stock_value / 3.0);
        let vo_value =
            (total_value * sub_allocations.us_stock_mid / 100.0) - (other_us_stock_value / 3.0);
        let vb_value =
            (total_value * sub_allocations.us_stock_small / 100.0) - (other_us_stock_value / 3.0);
        let vtc_value =
            (total_value * sub_allocations.us_corp_bond / 100.0) - (other_us_bond_value / 2.0);
        let vv_value =
            (total_value * sub_allocations.us_stock_large / 100.0) - (other_us_stock_value / 3.0);
        let vtip_value = total_value * sub_allocations.inflation_protected / 100.0;

        // set vmfxx, ie cash, target value to 0 and return ShareValues
        ShareValues {
            vxus: vxus_value,
            bndx: bndx_value,
            bnd: bnd_value,
            vwo: vwo_value,
            vo: vo_value,
            vb: vb_value,
            vtc: vtc_value,
            vv: vv_value,
            vtip: vtip_value,
            other: 0.0,
            vmfxx: 0.0,
            outside_bond: other_int_bond_value + other_us_bond_value,
            outside_stock: other_us_stock_value + other_int_stock_value,
        }
    }

    /// add_stockinfo_value adds stock value to the ShareValues struct with a StockInfo input.  StockInfo
    /// structs are constructed when parsing the CSV file downloaded from vangaurd.  This is used
    /// for both creating the stock quotes ShareValues struct and holding values ShareValuues
    /// struc.  The add_type is used to distinguish between these two groups to know where from
    /// within the StockInfo struct to pull the dollar amount from.
    ///
    /// # Panic
    ///
    /// Panics when an empty stock symbol is passed.  This will happen if the StockInfo struct is
    /// initialized without any content added.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    /// new_stock.add_total_value(5000.00);
    ///
    /// let mut new_quotes = holdings::ShareValues::new_quote();
    /// new_quotes.add_stockinfo_value(new_stock, holdings::AddType::StockPrice);
    ///
    /// assert_eq!(new_quotes.stock_value(holdings::StockSymbol::BND), 234.50);
    ///
    /// ```
    pub fn add_stockinfo_value(&mut self, stock_info: StockInfo, add_type: AddType) {
        let value = match add_type {
            AddType::StockPrice => stock_info.share_price,
            AddType::HoldingValue => stock_info.total_value,
            AddType::HoldingShares => stock_info.shares,
        };
        match stock_info.symbol {
            StockSymbol::VXUS => self.vxus = value,
            StockSymbol::BNDX => self.bndx = value,
            StockSymbol::VTIP => self.vtip = value,
            StockSymbol::BND => self.bnd = value,
            StockSymbol::VWO => self.vwo = value,
            StockSymbol::VO => self.vo = value,
            StockSymbol::VB => self.vb = value,
            StockSymbol::VTC => self.vtc = value,
            StockSymbol::VV => self.vv = value,
            StockSymbol::VMFXX => self.vmfxx = value,
            StockSymbol::Empty => panic!("Stock symbol not set before adding value"),
            StockSymbol::Other(_) => match add_type {
                AddType::HoldingValue => self.other += value,
                AddType::StockPrice => self.other = 1.0,
                AddType::HoldingShares => self.other = 1.0,
            },
        }
    }

    /// add_stock_value adds stock value to the ShareValues struct with a float.  
    ///
    /// # Panic
    ///
    /// Panics when an empty stock symbol is passed.  This will happen if the StockInfo struct is
    /// initialized without any content added.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_values = holdings::ShareValues::new();
    /// new_values.add_stock_value(holdings::StockSymbol::BND, 5000.0);
    ///
    /// assert_eq!(new_values.stock_value(holdings::StockSymbol::BND), 5000.0);
    ///
    /// ```
    pub fn add_stock_value(&mut self, stock_symbol: StockSymbol, value: f32) {
        match stock_symbol {
            StockSymbol::VXUS => self.vxus = value,
            StockSymbol::BNDX => self.bndx = value,
            StockSymbol::VTIP => self.vtip = value,
            StockSymbol::BND => self.bnd = value,
            StockSymbol::VWO => self.vwo = value,
            StockSymbol::VO => self.vo = value,
            StockSymbol::VB => self.vb = value,
            StockSymbol::VTC => self.vtc = value,
            StockSymbol::VV => self.vv = value,
            StockSymbol::VMFXX => self.vmfxx = value,
            StockSymbol::Empty => panic!("Stock symbol not set before adding value"),
            StockSymbol::Other(_) => self.other = value,
        }
    }

    /// Adds other stock value that is not included within the vanguard account.  This is used for
    /// calculating current stock/bond ratios
    pub fn add_outside_stock_value(&mut self, stock_value: f32) {
        self.outside_stock = stock_value
    }

    pub fn outside_stock_value(&self) -> f32 {
        self.outside_stock
    }

    /// Adds other bond value that is not included within the vanguard account.  This is used for
    /// calculating current stock/bond ratios
    pub fn add_outside_bond_value(&mut self, bond_value: f32) {
        self.outside_bond = bond_value
    }

    pub fn outside_bond_value(&self) -> f32 {
        self.outside_bond
    }

    pub fn subtract_stock_value(&mut self, stock_symbol: StockSymbol, value: f32) {
        match stock_symbol {
            StockSymbol::VXUS => self.vxus -= value,
            StockSymbol::BNDX => self.bndx -= value,
            StockSymbol::VTIP => self.vtip -= value,
            StockSymbol::BND => self.bnd -= value,
            StockSymbol::VWO => self.vwo -= value,
            StockSymbol::VO => self.vo -= value,
            StockSymbol::VB => self.vb -= value,
            StockSymbol::VTC => self.vtc -= value,
            StockSymbol::VV => self.vv -= value,
            StockSymbol::VMFXX => self.vmfxx -= value,
            StockSymbol::Empty => panic!("Stock symbol not set before adding value"),
            StockSymbol::Other(_) => self.other -= value,
        }
    }

    /// stock_value retrieves the stored stock value within the ShareValues struct
    ///
    /// # Panic
    ///
    /// Panics when an empty stock symbol is passed.  This will happen if the StockInfo struct is
    /// initialized without any content added.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_values = holdings::ShareValues::new();
    /// new_values.add_stock_value(holdings::StockSymbol::BND, 5000.0);
    ///
    /// assert_eq!(new_values.stock_value(holdings::StockSymbol::BND), 5000.0);
    ///
    /// ```
    pub fn stock_value(&self, stock_symbol: StockSymbol) -> f32 {
        match stock_symbol {
            StockSymbol::VXUS => self.vxus,
            StockSymbol::BNDX => self.bndx,
            StockSymbol::VTIP => self.vtip,
            StockSymbol::BND => self.bnd,
            StockSymbol::VWO => self.vwo,
            StockSymbol::VO => self.vo,
            StockSymbol::VB => self.vb,
            StockSymbol::VTC => self.vtc,
            StockSymbol::VV => self.vv,
            StockSymbol::VMFXX => self.vmfxx,
            StockSymbol::Empty => panic!("Value retrieval not supported for empty stock symbol"),
            StockSymbol::Other(_) => self.other,
        }
    }

    /// total_value returns the sum of all of the values within the StockValue struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_values = holdings::ShareValues::new();
    /// new_values.add_stock_value(holdings::StockSymbol::BND, 5000.0);
    /// new_values.add_stock_value(holdings::StockSymbol::BNDX, 2000.0);
    /// new_values.add_stock_value(holdings::StockSymbol::VB, 4000.0);
    ///
    /// assert_eq!(new_values.total_value(), 11000.0);
    ///
    /// ```
    pub fn total_value(&self) -> f32 {
        self.vxus
            + self.bndx
            + self.bnd
            + self.vwo
            + self.vo
            + self.vb
            + self.vtc
            + self.vv
            + self.vmfxx
            + self.vtip
            + self.other
    }

    /// percent_stock_bond_infl calculates the percent of stock, bond, and inflation protected
    /// assets within the ShareValues.  This should only be used when the struct contains dollar
    /// value amounts for the stock values.
    pub fn percent_stock_bond_infl(&self) -> (f32, f32, f32) {
        let total_bond = self.bndx + self.bnd + self.vtc + self.outside_bond;
        let total_stock = self.vwo + self.vo + self.vb + self.vv + self.vxus + self.outside_stock;
        let total =
            self.total_value() - self.vmfxx - self.other + self.outside_bond + self.outside_stock;
        (
            total_stock / total * 100.0,
            total_bond / total * 100.0,
            self.vtip / total * 100.0,
        )
    }
}

impl Default for ShareValues {
    fn default() -> Self {
        Self::new()
    }
}

impl Add for ShareValues {
    type Output = ShareValues;

    fn add(self, other: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus + other.vxus,
            bndx: self.bndx + other.bndx,
            vtip: self.vtip + other.vtip,
            bnd: self.bnd + other.bnd,
            vwo: self.vwo + other.vwo,
            vo: self.vo + other.vo,
            vb: self.vb + other.vb,
            vtc: self.vtc + other.vtc,
            vv: self.vv + other.vv,
            vmfxx: self.vmfxx + other.vmfxx,
            other: self.other + other.other,
            outside_bond: self.outside_bond + other.outside_bond,
            outside_stock: self.outside_stock + other.outside_stock,
        }
    }
}

impl Sub for ShareValues {
    type Output = ShareValues;

    fn sub(self, other: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus - other.vxus,
            bndx: self.bndx - other.bndx,
            vtip: self.vtip - other.vtip,
            bnd: self.bnd - other.bnd,
            vwo: self.vwo - other.vwo,
            vo: self.vo - other.vo,
            vb: self.vb - other.vb,
            vtc: self.vtc - other.vtc,
            vv: self.vv - other.vv,
            vmfxx: self.vmfxx - other.vmfxx,
            other: self.other - other.other,
            outside_bond: self.outside_bond - other.outside_bond,
            outside_stock: self.outside_stock - other.outside_stock,
        }
    }
}

impl Div for ShareValues {
    type Output = ShareValues;

    fn div(self, other: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus / other.vxus,
            bndx: self.bndx / other.bndx,
            vtip: self.vtip / other.vtip,
            bnd: self.bnd / other.bnd,
            vwo: self.vwo / other.vwo,
            vo: self.vo / other.vo,
            vb: self.vb / other.vb,
            vtc: self.vtc / other.vtc,
            vv: self.vv / other.vv,
            vmfxx: self.vmfxx / other.vmfxx,
            other: self.other / other.other,
            outside_bond: self.outside_bond / other.outside_bond,
            outside_stock: self.outside_stock / other.outside_stock,
        }
    }
}

impl Mul for ShareValues {
    type Output = ShareValues;

    fn mul(self, other: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus * other.vxus,
            bndx: self.bndx * other.bndx,
            vtip: self.vtip * other.vtip,
            bnd: self.bnd * other.bnd,
            vwo: self.vwo * other.vwo,
            vo: self.vo * other.vo,
            vb: self.vb * other.vb,
            vtc: self.vtc * other.vtc,
            vv: self.vv * other.vv,
            vmfxx: self.vmfxx * other.vmfxx,
            other: self.other * other.other,
            outside_bond: self.outside_bond * other.outside_bond,
            outside_stock: self.outside_stock * other.outside_stock,
        }
    }
}

impl fmt::Display for ShareValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (stock, bond, inflation) = self.percent_stock_bond_infl();
        write!(
            f,
            "\
            Symbol         Value\n\
            -------------------------------\n\
            VV               {:.2}\n\
            VO               {:.2}\n\
            VB               {:.2}\n\
            VTC              {:.2}\n\
            BND              {:.2}\n\
            VXUS             {:.2}\n\
            VWO              {:.2}\n\
            BNDX             {:.2}\n\
            VTIP             {:.2}\n\
            -------------------------------\n\
            Cash             {:.2}\n\
            Total            {:.2}\n\
            Outside stock    {:.2}\n\
            Outside bond     {:.2}\n\
            Stock:Bond:Infl  {:.1}:{:.1}:{:.1}\n\
            ===============================
            ",
            self.vv,
            self.vo,
            self.vb,
            self.vtc,
            self.bnd,
            self.vxus,
            self.vwo,
            self.bndx,
            self.vtip,
            self.vmfxx,
            self.total_value(),
            self.outside_stock,
            self.outside_bond,
            stock,
            bond,
            inflation
        )
    }
}

pub enum HoldingType {
    Brokerage,
    TraditionalIra,
    RothIra,
}

/// VanguardHoldings contains ShareValues structs for all accounts along with for the quotes.  This
/// struct is creating during the parsing of the downloaded Vanguard file
#[derive(Clone, Debug)]
pub struct VanguardHoldings {
    pub accounts_values: HashMap<u32, ShareValues>,
    pub accounts_shares: HashMap<u32, ShareValues>,
    quotes: ShareValues,
    transactions: Vec<Transaction>, // holds all transactions, which needs to be filtered by trad
    // acct num later
    distributions: HashMap<u32, f32>,
}

impl VanguardHoldings {
    /// new creates a new VanguardHoldings struct with the quotes added.  The rest of the accounts
    /// needs to be added later
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let new_quotes = holdings::ShareValues::new_quote();
    ///
    /// let mut new_vanguard = holdings::VanguardHoldings::new(new_quotes);
    /// ```
    pub fn new(quotes: ShareValues) -> Self {
        VanguardHoldings {
            accounts_values: HashMap::new(),
            accounts_shares: HashMap::new(),
            quotes,
            transactions: Vec::new(),
            distributions: HashMap::new(),
        }
    }

    pub fn stock_quotes(&self) -> ShareValues {
        self.quotes
    }
    pub fn transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }
    pub fn distributions(&self, account_number: &u32) -> f32 {
        self.distributions
            .get(account_number)
            .unwrap_or(&0.0)
            .clone()
    }
    // Calculated the previous end of year holdings value based on the holdings times the quotes
    // from December 31st of the previous year.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn eoy_value(&mut self, year: u32, traditional_acct_num: u32) -> Result<Option<f32>> {
        let trad_holdings = self
            .accounts_values
            .get(&traditional_acct_num)
            .unwrap_or(&ShareValues::new())
            .clone();
        if let Some(holdings) =
            self.eoy_traditional_holdings(year, traditional_acct_num, trad_holdings)
        {
            let mut quotes = ShareValues::new_quote();
            quotes.add_missing_eoy_quotes(year - 1).await?;
            let eoy_value = (holdings * quotes).total_value();
            Ok(Some(eoy_value))
        } else {
            Ok(None)
        }
    }
    // Takes the current holdings and subtracts all transaction since December 31st to come to the
    // holdings at that date.
    #[cfg(not(target_arch = "wasm32"))]
    fn eoy_traditional_holdings(
        &mut self,
        year: u32,
        traditional_acct_num: u32,
        trad_holdings: ShareValues,
    ) -> Option<ShareValues> {
        let mut enough_transaction = false;
        let mut total_transactions = 0;
        let mut eoy_holdings = trad_holdings;
        let previous_year = NaiveDate::from_ymd_opt(year as i32 - 1, 12, 31)?;
        let following_year = previous_year + Duration::days(365);
        for transaction in &self.transactions {
            // If the transaction is newer thand December 31st of the previous year,
            // subtract from the current holdings.  Also stores a true value if anything is
            // older to keep track whether or not enough transactions were pulled from
            // Vanguard to get to December 31st.
            if transaction.trade_date > previous_year
                && transaction.account_number == traditional_acct_num
            {
                total_transactions += 1;
                // Cash is allocated in VMFXX.  These are not shares in the transaction, so
                // net amount needs to be subtracted
                if transaction.symbol == StockSymbol::VMFXX {
                    eoy_holdings
                        .subtract_stock_value(transaction.symbol.clone(), transaction.net_amount);
                } else if transaction.symbol != StockSymbol::Empty {
                    eoy_holdings
                        .subtract_stock_value(transaction.symbol.clone(), transaction.shares);
                } else if transaction.transaction_type == TransactionType::DISTRIBUTION {
                    if transaction.trade_date < following_year {
                        let distribution = self
                            .distributions
                            .entry(transaction.account_number)
                            .or_insert(0.0);
                        *distribution -= transaction.net_amount;
                    }
                }
            } else {
                enough_transaction = true;
            }
        }
        if !enough_transaction {
            None
        } else if total_transactions == 0 {
            None
        } else {
            Some(eoy_holdings)
        }
    }
}

impl Default for VanguardHoldings {
    fn default() -> Self {
        VanguardHoldings::new(ShareValues::new_quote())
    }
}

/// AccountHoldings is a holder of current, target, and purchase/sales information for an account.
/// It also creates a Display for this information.
#[derive(Debug)]
pub struct AccountHoldings {
    pub current: ShareValues,
    pub target: ShareValues,
    pub sale_purchases_needed: ShareValues,
}

impl AccountHoldings {
    /// new creates a new AccountHoldings struct from current, target, and sales/purchases
    /// Sharevalues structs.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::{asset, holdings};
    ///
    /// let sub_allocations = asset::SubAllocations::new().unwrap();
    ///
    /// let quotes = holdings::ShareValues::new_quote();
    ///
    /// let brokerage_current = holdings::ShareValues::new();
    /// let brokerage_target = holdings::ShareValues::new_target(sub_allocations, 10000.0, 0.0, 0.0, 0.0, 0.0);
    /// let purchase_sales = brokerage_current / quotes;
    ///
    /// let brokerage_account = holdings::AccountHoldings::new(brokerage_current, brokerage_target, purchase_sales);
    /// ```
    pub fn new(
        current: ShareValues,
        target: ShareValues,
        sale_purchases_needed: ShareValues,
    ) -> Self {
        AccountHoldings {
            current,
            target,
            sale_purchases_needed,
        }
    }
}

impl Default for AccountHoldings {
    fn default() -> Self {
        AccountHoldings {
            current: ShareValues::new(),
            target: ShareValues::new(),
            sale_purchases_needed: ShareValues::new(),
        }
    }
}

impl fmt::Display for AccountHoldings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (current_stock, current_bond, current_inflation) =
            self.current.percent_stock_bond_infl();
        let current_stock_bond = format!(
            "{:.1}:{:.1}:{:.1}",
            current_stock, current_bond, current_inflation
        );

        let (target_stock, target_bond, target_inflation) = self.target.percent_stock_bond_infl();
        let target_stock_bond = format!(
            "{:.1}:{:.1}:{:.1}",
            target_stock, target_bond, target_inflation
        );

        write!(
            f,
            "Symbol   Purchase/Sell  Current         Target\n\
            ------------------------------------------------------\n\
            VV       {:<15.2}${:<15.2}${:<15.2}\n\
            VO       {:<15.2}${:<15.2}${:<15.2}\n\
            VB       {:<15.2}${:<15.2}${:<15.2}\n\
            VTC      {:<15.2}${:<15.2}${:<15.2}\n\
            BND      {:<15.2}${:<15.2}${:<15.2}\n\
            VXUS     {:<15.2}${:<15.2}${:<15.2}\n\
            VWO      {:<15.2}${:<15.2}${:<15.2}\n\
            BNDX     {:<15.2}${:<15.2}${:<15.2}\n\
            VTIP     {:<15.2}${:<15.2}${:<15.2}\n\
            ------------------------------------------------------\n\
            Cash                    ${:<15.2}${:<15.2}\n\
            Total                   ${:<15.2}\n\
            Outside stock           ${:<15.2}${:<15.2}\n\
            Outside bond            ${:<15.2}${:<15.2}\n\
            Stock:Bond:Inflation    {:<16}{:<15}\n\
            ======================================================",
            self.sale_purchases_needed.vv,
            self.current.vv,
            self.target.vv,
            self.sale_purchases_needed.vo,
            self.current.vo,
            self.target.vo,
            self.sale_purchases_needed.vb,
            self.current.vb,
            self.target.vb,
            self.sale_purchases_needed.vtc,
            self.current.vtc,
            self.target.vtc,
            self.sale_purchases_needed.bnd,
            self.current.bnd,
            self.target.bnd,
            self.sale_purchases_needed.vxus,
            self.current.vxus,
            self.target.vxus,
            self.sale_purchases_needed.vwo,
            self.current.vwo,
            self.target.vwo,
            self.sale_purchases_needed.bndx,
            self.current.bndx,
            self.target.bndx,
            self.sale_purchases_needed.vtip,
            self.current.vtip,
            self.target.vtip,
            self.current.vmfxx,
            self.target.vmfxx,
            self.current.total_value(),
            self.current.outside_stock,
            self.target.outside_stock,
            self.current.outside_bond,
            self.target.outside_bond,
            current_stock_bond,
            target_stock_bond,
        )
    }
}

/// VanguardRebalance holds AccountHoldings structs for each account; brokerage, traditional IRA,
/// and roth IRA.  Each AccountHoldings struct holds the information of current holdings, target
/// holdings, and the amount of stocks needed to purchase/sell in order to rebalance
#[derive(Debug)]
pub struct VanguardRebalance {
    pub brokerage: AccountHoldings,
    pub traditional_ira: AccountHoldings,
    pub roth_ira: AccountHoldings,
    retirement_target: ShareValues,
}

impl VanguardRebalance {
    /// new creates a new empty VanguardRebalance struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let vanguard_rebalance = holdings::VanguardRebalance::new();
    /// ```
    pub fn new() -> Self {
        VanguardRebalance {
            brokerage: AccountHoldings::default(),
            traditional_ira: AccountHoldings::default(),
            roth_ira: AccountHoldings::default(),
            retirement_target: ShareValues::default(),
        }
    }

    /// add_account_holdings adds either roth IRA, traditional IRA, or brokerage AccountHoldings
    /// struct to the current VanguardRebalance struct.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::{asset, holdings};
    ///
    /// let quotes = holdings::ShareValues::new_quote();
    ///
    /// let sub_allocations = asset::SubAllocations::new().unwrap();
    ///
    /// let brokerage_current = holdings::ShareValues::new();
    /// let brokerage_target = holdings::ShareValues::new_target(sub_allocations, 10000.0, 0.0, 0.0, 0.0, 0.0);
    /// let purchase_sales = brokerage_current / quotes;
    ///
    /// let brokerage_account = holdings::AccountHoldings::new(brokerage_current, brokerage_target, purchase_sales);
    ///
    /// let mut vanguard_rebalance = holdings::VanguardRebalance::new();
    /// vanguard_rebalance.add_account_holdings(brokerage_account, holdings::HoldingType::Brokerage);
    /// ```
    pub fn add_account_holdings(&mut self, acct_holding: AccountHoldings, acct_type: HoldingType) {
        match acct_type {
            HoldingType::Brokerage => self.brokerage = acct_holding,
            HoldingType::TraditionalIra => self.traditional_ira = acct_holding,
            HoldingType::RothIra => self.roth_ira = acct_holding,
        }
    }

    pub fn add_retirement_target(&mut self, retirement_target: ShareValues) {
        self.retirement_target = retirement_target;
    }
}

impl Default for VanguardRebalance {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VanguardRebalance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out_string = String::new();
        out_string.push_str(&format!(
            "Retirement target:\n{}\n\n",
            self.retirement_target
        ));
        out_string.push_str(&format!("Traditional IRA:\n{}\n\n", self.traditional_ira));
        out_string.push_str(&format!("Roth IRA:\n{}\n\n", self.roth_ira));
        out_string.push_str(&format!("Brokerage:\n{}\n\n", self.brokerage));
        write!(f, "{}", out_string.trim_end_matches('\n'))
    }
}

#[derive(Clone, Debug)]
pub struct Transaction {
    account_number: u32,
    trade_date: NaiveDate,
    symbol: StockSymbol,
    shares: f32,
    net_amount: f32,
    transaction_type: TransactionType,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
enum TransactionType {
    CONVERSIONOUT,
    DIVIDEND,
    REINVESTMENT,
    ADVISORFEE,
    BUY,
    CONVERSIONIN,
    SELL,
    FUNDSRECEIVED,
    SWEEPOUT,
    SWEEPIN,
    DISTRIBUTION,
    Other(String),
}

impl TransactionType {
    /// new creates a new StockSymbol enum based on the string value.
    ///
    ///  # Example
    ///
    ///  ```
    ///  use vapore::holdings::TransactionType;
    ///
    ///  let div = TransactionType::new("Dividend");
    ///  assert_eq!(div, TransactionType::DIVIDEND);
    ///  ```
    pub fn new(transaction_type: &str) -> Self {
        match transaction_type {
            "Conversion (outgoing)" => TransactionType::CONVERSIONOUT,
            "Dividend" => TransactionType::DIVIDEND,
            "Reinvestment" => TransactionType::REINVESTMENT,
            "Advisor fee" => TransactionType::ADVISORFEE,
            "Buy" => TransactionType::BUY,
            "Conversion (incoming)" => TransactionType::CONVERSIONIN,
            "Sell" => TransactionType::SELL,
            "Funds Received" => TransactionType::FUNDSRECEIVED,
            "Sweep out" => TransactionType::SWEEPOUT,
            "Sweep in" => TransactionType::SWEEPIN,
            "Distribution" => TransactionType::DISTRIBUTION,
            _ => TransactionType::Other(transaction_type.to_string()),
        }
    }
}

/// parse_csv_download takes in the file path of the downloaded file from Vanguard and parses it
/// into VanguardHoldings.  The VanguardHoldings is a struct which holds the values of what is
/// contained within the vangaurd account along with quotes for each of the ETFs
pub async fn parse_csv_download(csv_string: String) -> Result<VanguardHoldings> {
    let mut header = Vec::new();
    let mut transaction_header = Vec::new();
    let mut accounts_values: HashMap<u32, ShareValues> = HashMap::new();
    let mut accounts_shares: HashMap<u32, ShareValues> = HashMap::new();
    let mut quotes = ShareValues::new_quote();

    let mut holdings_row = true;
    let mut transactions = Vec::new();

    // iterate through all of the rows of the vanguard downlaoaded file and add the information to
    // StockInfo structs, which then are aggregated into the accounts hashmap where the account
    // number is the key
    for row in csv_string.split('\n') {
        println!("{}", row);
        if row.contains(',') {
            if row.contains("Trade Date") {
                holdings_row = false;
            }
            let row_split = row
                .split(',')
                .map(|value| value.to_string())
                .collect::<Vec<String>>();
            if row_split.len() > 4 {
                if holdings_row {
                    let mut stock_info = StockInfo::new();
                    if header.is_empty() {
                        header = row_split
                    } else {
                        for (value, head) in row_split.iter().zip(&header) {
                            match head.as_str() {
                                "Account Number" => stock_info.add_account(value.parse::<u32>()?),
                                "Symbol" => {
                                    if value.chars().count() > 1 {
                                        stock_info.add_symbol(StockSymbol::new(value))
                                    } else {
                                        break;
                                    }
                                }
                                "Shares" => stock_info.add_shares(value.parse::<f32>()?),
                                "Share Price" => stock_info.add_share_price(value.parse::<f32>()?),
                                "Total Value" => stock_info.add_total_value(value.parse::<f32>()?),
                                _ => continue,
                            }
                        }
                        if stock_info.finished() {
                            let account_value = accounts_values
                                .entry(stock_info.account_number)
                                .or_insert_with(ShareValues::new);
                            account_value
                                .add_stockinfo_value(stock_info.clone(), AddType::HoldingValue);
                            let account_shares = accounts_shares
                                .entry(stock_info.account_number)
                                .or_insert_with(ShareValues::new);
                            account_shares
                                .add_stockinfo_value(stock_info.clone(), AddType::HoldingShares);
                            quotes.add_stockinfo_value(stock_info.clone(), AddType::StockPrice);
                        }
                    }
                } else if transaction_header.is_empty() {
                    transaction_header = row_split
                } else {
                    let mut account_num_option = None;
                    let mut trade_date_option = None;
                    let mut symbol_option = None;
                    let mut shares_option = None;
                    let mut net_amount_option = None;
                    let mut transaction_type_option = None;
                    for (value, head) in row_split.iter().zip(&transaction_header) {
                        match head.as_str() {
                            "Account Number" => account_num_option = Some(value.parse::<u32>()?),
                            "Symbol" => symbol_option = Some(StockSymbol::new(value)),
                            "Shares" => shares_option = Some(value.parse::<f32>()?),
                            "Trade Date" => {
                                trade_date_option =
                                    Some(NaiveDate::parse_from_str(value, "%Y-%m-%d")?)
                            }
                            "Net Amount" => net_amount_option = Some(value.parse::<f32>()?),
                            "Transaction Type" => {
                                transaction_type_option = Some(TransactionType::new(value))
                            }
                            _ => continue,
                        }
                    }
                    if let Some(account_number) = account_num_option {
                        if let Some(symbol) = symbol_option {
                            if let Some(shares) = shares_option {
                                if let Some(trade_date) = trade_date_option {
                                    if let Some(net_amount) = net_amount_option {
                                        if let Some(transaction_type) = transaction_type_option {
                                            transactions.push(Transaction {
                                                account_number: account_number,
                                                symbol,
                                                shares,
                                                trade_date,
                                                net_amount,
                                                transaction_type,
                                            })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    quotes.add_missing_quotes().await?;

    Ok(VanguardHoldings {
        accounts_values,
        accounts_shares,
        quotes,
        transactions,
        distributions: HashMap::new(),
    })
}
