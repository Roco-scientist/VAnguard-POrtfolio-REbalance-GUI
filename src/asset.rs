use anyhow::{ensure, Result};
use chrono::prelude::*;
use std::fmt;

// Constants used for proportion of portfolio contained within each.
// Split by stocks and bonds
// US stock as 2/3 of total stock.  Then split by 3 for Large, medium, and small cap
const US_STOCK_FRACTION: f32 = 2.0 / 3.0;
const EACH_US_STOCK: f32 = US_STOCK_FRACTION / 3.0;
// International stock as 1/3 of total stock.  Then 1/3 of that as emerging markets and 2/3 as
// total international
const INT_STOCK_FRACTION: f32 = 1.0 / 3.0;
const INT_EMERGING: f32 = INT_STOCK_FRACTION / 3.0;
const INT_TOTAL: f32 = INT_STOCK_FRACTION * 2.0 / 3.0;
// 2/3 of total bonds in US corporate bonds, 1/3 in internation bonds
const US_BOND_FRACTION: f32 = 2.0 / 3.0;
const US_CORP_BOND_FRACTION: f32 = US_BOND_FRACTION / 2.0;
const US_TOT_BOND_FRACTION: f32 = US_BOND_FRACTION / 2.0;
const INT_BOND_FRACTION: f32 = 1.0 / 3.0;

/// Holds the stock, bond, and inflation protected percentages.
pub struct Allocations {
    total_stock: f32,
    total_bond: f32,
    total_inflation_protected: f32,
}

impl Allocations {
    /// Default asset allocations at 60% stock and 40% bond
    pub fn new() -> Self {
        Allocations {
            total_stock: 60.0,
            total_bond: 40.0,
            total_inflation_protected: 0.0,
        }
    }
    /// Calculates the stock, bond, and inflation protected percentages based on Vanguard target
    /// asset allocation.
    pub fn retirement(year: i32) -> Result<Self> {
        ensure!(
            (2000..3000).contains(&year),
            format!(
                "Year needs to be between 2000 and 3000.  Year input: {}",
                year
            )
        );
        let this_year = chrono::Local::now().year();
        let years_to_retirement = (year - this_year) as f32;
        let mut total_stock = 90.0;
        let mut total_bond = 10.0;
        let mut total_inflation_protected = 0.0;
        if (5.0..30.0).contains(&years_to_retirement) {
            total_stock = 90.0 - (1.5 * (25.0 - years_to_retirement));
            total_bond = 100.0 - total_stock;
        } else if (-5.0..5.0).contains(&years_to_retirement) {
            total_stock = 60.0 - (-2.8 * (years_to_retirement - 5.0));
            total_inflation_protected = -1.8 * (years_to_retirement - 5.0);
            total_bond = 100.0 - total_stock - total_inflation_protected;
        } else if years_to_retirement < -5.0 {
            total_stock = 29.0;
            total_bond = 53.0;
            total_inflation_protected = 18.0;
        }
        Ok(Allocations {
            total_stock,
            total_bond,
            total_inflation_protected,
        })
    }

    /// Creates a Allocations struct with custom input values for stock, bond, and inflaction
    /// protected precentages.
    pub fn custom(
        total_stock: f32,
        total_bond: f32,
        total_inflation_protected: f32,
    ) -> Result<Self> {
        ensure!(
            total_stock + total_bond + total_inflation_protected == 100.0,
            format!(
                "Stock ({}) + bond ({}) + inflation protected ({}) does not equal 100",
                total_stock, total_bond, total_inflation_protected
            )
        );
        Ok(Allocations {
            total_stock,
            total_bond,
            total_inflation_protected,
        })
    }

    /// Return total stock asset allocation percentage.
    pub fn total_stock(&self) -> f32 {
        self.total_stock
    }

    /// Returns total bond asset allocation percentage.
    pub fn total_bond(&self) -> f32 {
        self.total_bond
    }

    /// Returns total inflation protected asset allocation percentage.
    pub fn total_inflation_protected(&self) -> f32 {
        self.total_inflation_protected
    }
}

impl Default for Allocations {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Allocations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Percent stock: {:.2}\nPercent bond: {:.2}\nPercent inflation protected: {:.2}",
            self.total_stock, self.total_bond, self.total_inflation_protected
        )
    }
}

/// Holds the percentage of allocation for each type of stock, bond, etc.  splitting by US and
/// international and other categories.
pub struct SubAllocations {
    pub us_stock_large: f32,
    pub us_stock_mid: f32,
    pub us_stock_small: f32,
    pub us_tot_bond: f32,
    pub us_corp_bond: f32,
    pub int_tot_stock: f32,
    pub int_emerging_stock: f32,
    pub int_bond: f32,
    pub inflation_protected: f32,
}

impl SubAllocations {
    /// Creates a default SubAllocations struct using the default Allocations of 60% stock and 40%
    /// bond
    pub fn new() -> Result<Self> {
        let allocations = Allocations::new();
        Self::new_custom(allocations)
    }

    /// Divides the asset bond/stock allocations set by the Allocations struct into percentages for
    /// the SubAllocations of how much within international, domestic, bond, stock etc.
    pub fn new_custom(allocations: Allocations) -> Result<Self> {
        let us_stock_large = allocations.total_stock() * EACH_US_STOCK;
        let us_stock_mid = allocations.total_stock() * EACH_US_STOCK;
        let us_stock_small = allocations.total_stock() * EACH_US_STOCK;
        let us_tot_bond = allocations.total_bond() * US_TOT_BOND_FRACTION;
        let us_corp_bond = allocations.total_bond() * US_CORP_BOND_FRACTION;
        let int_tot_stock = allocations.total_stock() * INT_TOTAL;
        let int_emerging_stock = allocations.total_stock() * INT_EMERGING;
        let int_bond = allocations.total_bond() * INT_BOND_FRACTION;
        let inflation_protected = allocations.total_inflation_protected();
        let sum = us_stock_large
            + us_stock_mid
            + us_stock_small
            + us_tot_bond
            + us_corp_bond
            + int_tot_stock
            + int_emerging_stock
            + int_bond
            + inflation_protected;
        ensure!(
            (99.9..100.1).contains(&sum),
            format!("Total sub allocations did not add up to 100: {}", sum)
        );
        Ok(SubAllocations {
            us_stock_large,
            us_stock_mid,
            us_stock_small,
            us_tot_bond,
            us_corp_bond,
            int_tot_stock,
            int_emerging_stock,
            int_bond,
            inflation_protected,
        })
    }
}
