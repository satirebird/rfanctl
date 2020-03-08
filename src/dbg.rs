use cortex_m::{
    peripheral::ITM,
    peripheral::DCB,
    peripheral::TPIU,
    itm,
};
use core::{ptr, fmt};
use stm32f1xx_hal::rcc::Clocks;
use stm32f1xx_hal::time::Hertz;

//use core::convert::Infallible;
//use ufmt::uWrite;

static mut DBG_INST: Option<Debug> = None;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::dbg::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::dbg::_write("\n"));
    ($($arg:tt)*) => ({
        $crate::dbg::_print(format_args!($($arg)*));
        $crate::dbg::_write("\n");
    })
}

pub fn _print(args: fmt::Arguments<'_>) {
    unsafe{
        let stim = &mut DBG_INST.as_mut().unwrap().itm.stim[0];
        itm::write_fmt(stim, args);
    }
}

pub fn _write(s: &str) {
    unsafe{
        let stim = &mut DBG_INST.as_mut().unwrap().itm.stim[0];
        itm::write_str(stim, s);
    }
}


// macro_rules! uprint {
//     // IMPORTANT use `tt` fragments instead of `expr` fragments (i.e. `$($exprs:expr),*`)
//     ($($tt:tt)*) => {{
//         let _ = ufmt::uwrite!($crate::dbg::itmout(), $($tt)*);
//     }}
// }


pub struct Debug {
    itm: ITM, 
    dcb: DCB, 
    tpiu: TPIU, 
}

// impl uWrite for Debug {
//     type Error = Infallible;

//     fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
//         let stim = &mut self.itm.stim[0];
//         itm::write_str(stim, s);
//         Ok(())
//     }
// }

// pub fn itmout() -> &'static mut Debug{
//     unsafe{
//         DBG_INST.as_mut().unwrap()
//     }
// }

impl Debug{

    pub fn init(itm: ITM, dcb: DCB, tpiu: TPIU){
        unsafe{
            DBG_INST = Some(Debug {
                itm: itm,
                dcb: dcb,
                tpiu: tpiu,
            });
        }        
    }

    // https://github.com/rust-embedded/cortex-m-semihosting/blob/master/src/export.rs

    pub fn enable_swo<F>(clocks: &Clocks, freq: F) where F: Into<Hertz>{                   
        
        unsafe {
            let inst = &DBG_INST.as_ref().unwrap();
            // enable TPIU and ITM
            inst.dcb.demcr.modify(|r| r | (1 << 24));
    
            // prescaler
            //let swo_freq = 2_000_000;
            let swo_freq = freq.into().0;
            inst.tpiu.acpr.write((clocks.sysclk().0 / swo_freq) - 1);
    
            // SWO NRZ
            inst.tpiu.sppr.write(2);
    
            inst.tpiu.ffcr.modify(|r| r & !(1 << 1));
    
            // STM32 specific: enable tracing in the DBGMCU_CR register
            const DBGMCU_CR: *mut u32 = 0xe0042004 as *mut u32;
            let r = ptr::read_volatile(DBGMCU_CR);
            ptr::write_volatile(DBGMCU_CR, r | (1 << 5));
    
            // unlock the ITM
            inst.itm.lar.write(0xC5ACCE55);
    
            inst.itm.tcr.write(
                (0b000001 << 16) | // TraceBusID
                (1 << 3) | // enable SWO output
                (1 << 0), // enable the ITM
            );
    
            // enable stimulus port 0
            inst.itm.ter[0].write(1);
        }
    }
}
