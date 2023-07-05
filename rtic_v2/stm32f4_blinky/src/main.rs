#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_rtt_target as _;
use rtic::app;
use rtic_monotonics::systick::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::*;
use stm32f4xx_hal::prelude::*;

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI1])]
mod app {
    use super::*;
    use rtic_sync::channel::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
        state: bool,
        button: PC13<Input>,
        tx: Sender<'static,usize,32>,
        rx: Receiver<'static,usize,32>,
    }

    #[init]
    fn init(mut cx: init::Context<'a>) -> (Shared, Local) {
        let gpioa = cx.device.GPIOA.split();
        let mut led = gpioa.pa5.into_push_pull_output();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = cx.device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 48_000_000, systick_mono_token); // default STM32F303 clock-rate is 36MHz

        let mut syscfg = cx.device.SYSCFG.constrain();
        let mut button = cx.device.GPIOC.split().pc13.into_pull_down_input();
        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut cx.device.EXTI);
        button.trigger_on_edge(&mut cx.device.EXTI, Edge::Falling);

        rtt_init_print!();
        rprintln!("init");

        // Setup LED
        led.set_high();

        let (tx, rx) = rtic_sync::make_channel!(usize,32);

        // Schedule the blinking task
        receiver::spawn().ok();
        blink::spawn().ok();

        (
            Shared {

            },
            Local {
                led,
                state: false,
                button,
                tx,
                rx,
            },
        )
    }

    /// React on the button click
    // see here for why this is EXTI15_10: https://github.com/stm32-rs/stm32f4xx-hal/blob/6d0c29233a4cd1f780b2fef3e47ef091ead6cf4a/src/gpio/exti.rs#L8-L23
    #[task(binds = EXTI15_10, local = [button, tx])]
    fn button_click(cx: button_click::Context) {
        cx.local.button.clear_interrupt_pending_bit();
        rprintln!("button!");
        cx.local.tx.try_send(9).unwrap();
    }

    #[task(local = [led, state])]
    async fn blink(cx: blink::Context) {
        loop {
            rprintln!("blink");
            //cortex_m_semihosting::hprintln!("blink!");
            if *cx.local.state {
                cx.local.led.set_high();
                *cx.local.state = false;
            } else {
                cx.local.led.set_low();
                *cx.local.state = true;
            }
            Systick::delay(1000.millis()).await;


        }
    }

    #[task(local = [rx])]
    async fn receiver(cx: receiver::Context) {
        rprintln!("reciever");
        loop {
            let r = cx.local.rx.recv().await.unwrap();
            rprintln!("Got: {}", r);
        }
    }

}
