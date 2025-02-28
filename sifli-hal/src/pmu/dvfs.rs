use crate::pac::hpsys_cfg::regs::Ulpmcr;
use crate::pac::{HPSYS_CFG, PMUC};
use crate::time::Hertz;

// Constants for DVFS mode limits
pub const HPSYS_DVFS_MODE_D0_LIMIT: u32 = 24;
pub const HPSYS_DVFS_MODE_D1_LIMIT: u32 = 48;
pub const HPSYS_DVFS_MODE_S0_LIMIT: u32 = 144;
pub const HPSYS_DVFS_MODE_S1_LIMIT: u32 = 240;

pub const HPSYS_DVFS_CONFIG: [HpsysDvfsConfig; 4] = [
    // LDO: 0.9V, BUCK: 1.0V
    HpsysDvfsConfig { ldo_offset: -5, ldo: 0x6, buck: 0x9, ulpmcr: 0x00100330 },
    // LDO: 1.0V, BUCK: 1.1V
    HpsysDvfsConfig { ldo_offset: -3, ldo: 0x8, buck: 0xA, ulpmcr: 0x00110331 },
    // LDO: 1.1V, BUCK: 1.25V
    HpsysDvfsConfig { ldo_offset:  0, ldo: 0xB, buck: 0xD, ulpmcr: 0x00130213 },
    // LDO: 1.2V, BUCK: 1.35V
    HpsysDvfsConfig { ldo_offset:  2, ldo: 0xD, buck: 0xF, ulpmcr: 0x00130213 },
];

pub const HPSYS_DLL2_LIMIT: [u32; 4] = [
    0,           // D0 Mode
    0,           // D1 Mode
    288_000_000, // S0 Mode
    288_000_000, // S1 Mode
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpsysDvfsMode {
    D0 = 0,
    D1 = 1,
    S0 = 2,
    S1 = 3,
}

#[cfg(feature = "defmt")]
impl defmt::Format for HpsysDvfsMode {
    fn format(&self, f: defmt::Formatter) {
        match self {
            HpsysDvfsMode::D0 => defmt::write!(f, "D0"),
            HpsysDvfsMode::D1 => defmt::write!(f, "D1"),
            HpsysDvfsMode::S0 => defmt::write!(f, "S0"),
            HpsysDvfsMode::S1 => defmt::write!(f, "S1"),
        }
    }
}

pub fn is_hpsys_dvfs_mode_s() -> bool {
    HPSYS_CFG.syscr().read().ldo_vsel()
}

impl HpsysDvfsMode {
    pub fn from_frequency(freq_mhz: u32) -> Result<Self, &'static str> {
        match freq_mhz {
            0..=HPSYS_DVFS_MODE_D0_LIMIT => Ok(HpsysDvfsMode::D0),
            25..=HPSYS_DVFS_MODE_D1_LIMIT => Ok(HpsysDvfsMode::D1),
            49..=HPSYS_DVFS_MODE_S0_LIMIT => Ok(HpsysDvfsMode::S0),
            145..=HPSYS_DVFS_MODE_S1_LIMIT => Ok(HpsysDvfsMode::S1),
            _ => Err("Frequency out of valid range"),
        }
    }

    pub fn from_hertz(freq: Hertz) -> Result<Self, &'static str> {
        Self::from_frequency(freq.0 / 1_000_000)
    }

    pub fn get_dll2_limit(self) -> Hertz {
        Hertz(HPSYS_DLL2_LIMIT[self as usize])
    }

    pub fn get_config(self) -> HpsysDvfsConfig {
        HPSYS_DVFS_CONFIG[self as usize]
    }

    pub fn get_frequency_limit(self) -> Hertz {
        Hertz(match self {
            HpsysDvfsMode::D0 => HPSYS_DVFS_MODE_D0_LIMIT,
            HpsysDvfsMode::D1 => HPSYS_DVFS_MODE_D1_LIMIT,
            HpsysDvfsMode::S0 => HPSYS_DVFS_MODE_S0_LIMIT,
            HpsysDvfsMode::S1 => HPSYS_DVFS_MODE_S1_LIMIT,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HpsysDvfsConfig {
    pub ldo_offset: i8,
    pub ldo: u8,
    pub buck: u8,
    pub ulpmcr: u32,
}

pub(crate) fn config_hcpu_dvfs<F>(
    current_hclk_freq: Hertz,
    target_hclk_freq: Hertz,
    config_clock: F,
) where
    F: FnOnce(),
{ 
    let current_dvfs_mode = HpsysDvfsMode::from_hertz(current_hclk_freq).unwrap();
    let target_dvfs_mode = HpsysDvfsMode::from_hertz(target_hclk_freq).unwrap();

    use HpsysDvfsMode::*;
    match (current_dvfs_mode, target_dvfs_mode) {
        (D0, D0) | (D1, D1) | (S0, S0) | (S1, S1) => config_clock(),
        (D0, S0) | (D1, S0) | (D1, S1) | (D0, S1) => switch_hcpu_dvfs_d2s(target_dvfs_mode, config_clock),
        (S0, D0) | (S0, D1) | (S1, D0) | (S1, D1) => switch_hcpu_dvfs_s2d(target_dvfs_mode, config_clock),
        (S0, S1) | (S1, S0) => {
            // switch between different S mode
            // TODO: HAL_RCC_HCPU_ClockSelect(RCC_CLK_MOD_SYS, RCC_SYSCLK_HRC48);

            config_hcpu_sx_mode_volt(target_dvfs_mode);
            // buck need 250us to settle
            crate::cortex_m_blocking_delay_us(250);
            config_clock();
        },
        (D0, D1) | (D1, D0) => {
            // TODO:
            // switch between different D mode
            // Why?
            // switch_hcpu_dvfs_d2s(target_dvfs_mode, todo_set_to_144);
            switch_hcpu_dvfs_s2d(target_dvfs_mode, config_clock);
        },
    }
}

fn config_hcpu_sx_mode_volt(target_dvfs_mode: HpsysDvfsMode) {
    let dvfs_config = target_dvfs_mode.get_config();

    // configure BUCK voltage
    PMUC.buck_vout().modify(| w| {
        w.set_vout(dvfs_config.buck);
    });

    // configure LDO voltage
    // TODO: use efuse value (HAL_PMU_GetHpsysVoutRef[2])
    let vout_ref = dvfs_config.ldo;
    PMUC.hpsys_vout().modify(| w| {
        w.set_vout(vout_ref);
    });
}

fn switch_hcpu_dvfs_d2s<F>(
    target_dvfs_mode: HpsysDvfsMode,
    config_clock: F,
) where
    F: FnOnce(),
{ 
    config_hcpu_sx_mode_volt(target_dvfs_mode);
    // Switch to S mode
    HPSYS_CFG.syscr().modify(|w| {
        w.set_ldo_vsel(false);
    });

    // buck need 250us to settle
    crate::cortex_m_blocking_delay_us(250);

    config_clock();
}

fn switch_hcpu_dvfs_s2d<F>(
    target_dvfs_mode: HpsysDvfsMode,
    config_clock: F,
) where
    F: FnOnce(),
{
    let dvfs_config = target_dvfs_mode.get_config();
    
    // configure BUCK voltage
    PMUC.buck_cr2().modify(| w| {
        w.set_set_vout_m(dvfs_config.buck);
    });

    // configure LDO voltage
    // TODO: use efuse value (HAL_PMU_GetHpsysVoutRef[2])
    let vout_ref = dvfs_config.ldo;
    PMUC.hpsys_ldo().modify(| w| {
        w.set_vref(vout_ref + dvfs_config.ldo_offset as u8);
    });

    config_clock();

    // configure memory param
    HPSYS_CFG.ulpmcr().write_value(Ulpmcr(dvfs_config.ulpmcr));
    // Switch to D mode
    HPSYS_CFG.syscr().modify(|w| {
        w.set_ldo_vsel(true);
    });
}
