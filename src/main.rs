#![no_std]
#![no_main]

// pick a panicking behavior
//extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

// https://github.com/rust-embedded/cortex-m/issues/82
// https://github.com/n-k/cortexm-threads

use nb::block;

use cortex_m::{
    asm,
};
use cortex_m_rt::entry;

use stm32f1xx_hal::{
    prelude::*,
    pac,
    timer::Timer,
};
use embedded_hal::digital::v2::OutputPin;

mod dbg;
use dbg::Debug;

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
 
    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
 
    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);
 
    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
 
    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(10.hz());    

    let mut dbg = Debug::new(cp.ITM, cp.DCB, cp.TPIU);
    dbg.enable_swo(&clocks, 2.mhz());
    dbg.tt();
    //enable_swo(cp, &clocks, 2.mhz());

    loop {
        block!(timer.wait()).unwrap();
        led.set_high().unwrap();
        block!(timer.wait()).unwrap();
        led.set_low().unwrap();
    }
}

/*
fn enable_swo<F>(cp: Peripherals, clocks: &Clocks, freq: F) 
    where F: Into<Hertz>
{
    unsafe {
        // enable TPIU and ITM
        cp.DCB.demcr.modify(|r| r | (1 << 24));

        // prescaler
        let swo_freq = 2_000_000;
        cp.TPIU.acpr.write((clocks.sysclk().0 / swo_freq) - 1);

        // SWO NRZ
        cp.TPIU.sppr.write(2);

        cp.TPIU.ffcr.modify(|r| r & !(1 << 1));

        // STM32 specific: enable tracing in the DBGMCU_CR register
        const DBGMCU_CR: *mut u32 = 0xe0042004 as *mut u32;
        let r = ptr::read_volatile(DBGMCU_CR);
        ptr::write_volatile(DBGMCU_CR, r | (1 << 5));

        // unlock the ITM
        cp.ITM.lar.write(0xC5ACCE55);

        cp.ITM.tcr.write(
            (0b000001 << 16) | // TraceBusID
            (1 << 3) | // enable SWO output
            (1 << 0), // enable the ITM
        );

        // enable stimulus port 0
        cp.ITM.ter[0].write(1);
    }
}
*/

// See here: https://docs.rs/svd2rust/0.16.1/svd2rust/#peripheral-api
// and https://docs.rs/stm32f1/0.8.0/stm32f1/stm32f103/index.html
// and https://docs.rs/stm32f1xx-hal/0.5.0/stm32f1xx_hal/rcc/struct.CFGR.html

