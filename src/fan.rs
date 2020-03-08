

use stm32f1xx_hal::{
    prelude::*,
    pac,
    timer::Timer,
    timer::Tim2NoRemap,
    pwm_input::*,
    pwm::*,
};

use stm32f1::stm32f103::*;
use stm32f1xx_hal::rcc::{Clocks, Rcc};
use stm32f1xx_hal::afio::Parts;

struct Fan{
    pwm_out: Pwm<TIM2, C1>,
}

impl Fan {
    pub fn new(&mut dp: Peripherals, &clocks : Clocks, &mut rcc: Rcc, &mut afio: Parts) -> Self{

        let tim = Timer::tim2(dp.TIM2, &clocks, &mut rcc.apb1);
        let pwm_out = tim.pwm::<Tim2NoRemap, _, _, _>(pins, &mut afio.mapr, 25.khz()).0;

        Fan{
            pwm_out,
        }
    }
}