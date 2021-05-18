/*!
Semi-private stuff that you usually don't need to access directly
 */

pub mod algorithm;
pub mod rotation;

mod aux_tables;
mod density_caching;
mod tables_wrapper;

#[cfg(test)]
mod unit_tests;
