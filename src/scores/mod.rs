//! Score modules. Each module is self-describing: it exposes a `DESCRIPTOR` carrying its
//! metadata, input contract, and compute function. [`ALL`] is the single source of truth the
//! registry and MCP tools derive from.

use crate::ScoreDescriptor;

pub mod apgar;
pub mod apls_weight;
pub mod cha2ds2_vasc;
pub mod child_pugh;
pub mod ckd_epi_2021;
pub mod cockcroft_gault;
pub mod crb_65;
pub mod curb_65;
pub mod fib_4;
pub mod gcs;
pub mod glasgow_blatchford;
pub mod has_bled;
pub mod meld_3;
pub mod meld_na;
pub mod mews;
pub mod news2;
pub mod padua_vte;
pub mod perc;
pub mod pews;
pub mod qsofa;
pub mod schwartz_egfr;
pub mod sirs;
pub mod sofa;
pub mod wells_pe;
pub mod westley_croup;

/// Every score Caliper exposes, in a stable order.
pub const ALL: &[ScoreDescriptor] = &[
    meld_na::DESCRIPTOR,
    meld_3::DESCRIPTOR,
    ckd_epi_2021::DESCRIPTOR,
    cockcroft_gault::DESCRIPTOR,
    schwartz_egfr::DESCRIPTOR,
    cha2ds2_vasc::DESCRIPTOR,
    has_bled::DESCRIPTOR,
    curb_65::DESCRIPTOR,
    crb_65::DESCRIPTOR,
    wells_pe::DESCRIPTOR,
    perc::DESCRIPTOR,
    news2::DESCRIPTOR,
    mews::DESCRIPTOR,
    qsofa::DESCRIPTOR,
    sirs::DESCRIPTOR,
    sofa::DESCRIPTOR,
    gcs::DESCRIPTOR,
    child_pugh::DESCRIPTOR,
    fib_4::DESCRIPTOR,
    glasgow_blatchford::DESCRIPTOR,
    padua_vte::DESCRIPTOR,
    // Pediatric / neonatal
    apgar::DESCRIPTOR,
    westley_croup::DESCRIPTOR,
    pews::DESCRIPTOR,
    apls_weight::DESCRIPTOR,
];
