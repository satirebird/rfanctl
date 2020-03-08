use cortex_m::{
    peripheral::ITM,
    peripheral::DCB,
    peripheral::TPIU,
    iprintln,
};
use core::ptr;
use stm32f1xx_hal::rcc::Clocks;
use stm32f1xx_hal::time::Hertz;

pub struct Debug {
    itm: ITM, 
    dcb: DCB, 
    tpiu: TPIU, 
}

impl Debug{
    pub fn new(itm: ITM, dcb: DCB, tpiu: TPIU) -> Self{
        Debug {
            itm: itm,
            dcb: dcb,
            tpiu: tpiu,
        }
    }

    pub fn tt(&mut self){
        let stim = &mut self.itm.stim[0];
        iprintln!(stim, "Hello, world!");        
    }

    // https://github.com/rust-embedded/cortex-m-semihosting/blob/master/src/export.rs

    pub fn enable_swo<F>(&self, clocks: &Clocks, freq: F) where F: Into<Hertz>{        
        unsafe {
            // enable TPIU and ITM
            self.dcb.demcr.modify(|r| r | (1 << 24));
    
            // prescaler
            //let swo_freq = 2_000_000;
            let swo_freq = freq.into().0;
            self.tpiu.acpr.write((clocks.sysclk().0 / swo_freq) - 1);
    
            // SWO NRZ
            self.tpiu.sppr.write(2);
    
            self.tpiu.ffcr.modify(|r| r & !(1 << 1));
    
            // STM32 specific: enable tracing in the DBGMCU_CR register
            const DBGMCU_CR: *mut u32 = 0xe0042004 as *mut u32;
            let r = ptr::read_volatile(DBGMCU_CR);
            ptr::write_volatile(DBGMCU_CR, r | (1 << 5));
    
            // unlock the ITM
            self.itm.lar.write(0xC5ACCE55);
    
            self.itm.tcr.write(
                (0b000001 << 16) | // TraceBusID
                (1 << 3) | // enable SWO output
                (1 << 0), // enable the ITM
            );
    
            // enable stimulus port 0
            self.itm.ter[0].write(1);
        }
    }
}

// pub fn init(itm: ITM, dcb: DCB, tpiu: TPIU, clocks: &Clocks){
//     unsafe {
//         // enable TPIU and ITM
//         dcb.demcr.modify(|r| r | (1 << 24));

//         // prescaler
//         let swo_freq = 2_000_000;
//         tpiu.acpr.write((clocks.sysclk().0 / swo_freq) - 1);

//         // SWO NRZ
//         tpiu.sppr.write(2);

//         tpiu.ffcr.modify(|r| r & !(1 << 1));

//         // STM32 specific: enable tracing in the DBGMCU_CR register
//         const DBGMCU_CR: *mut u32 = 0xe0042004 as *mut u32;
//         let r = ptr::read_volatile(DBGMCU_CR);
//         ptr::write_volatile(DBGMCU_CR, r | (1 << 5));

//         // unlock the ITM
//         itm.lar.write(0xC5ACCE55);

//         itm.tcr.write(
//             (0b000001 << 16) | // TraceBusID
//             (1 << 3) | // enable SWO output
//             (1 << 0), // enable the ITM
//         );

//         // enable stimulus port 0
//         itm.ter[0].write(1);
//     }

// }