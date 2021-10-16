use std::{ffi::c_void, thread, time::Duration};

use esp_idf_sys::{
    esp_err_t, gpio_num_t, rmt_carrier_level_t_RMT_CARRIER_LEVEL_HIGH,
    rmt_carrier_level_t_RMT_CARRIER_LEVEL_LOW, rmt_channel_t, rmt_config_t,
    rmt_config_t__bindgen_ty_1, rmt_mode_t_RMT_MODE_TX, rmt_tx_config_t, ESP_OK,
};
use log::*;

const WS2812_T0H_NS: u32 = 350;
const WS2812_T0L_NS: u32 = 1000;
const WS2812_T1H_NS: u32 = 1000;
const WS2812_T1L_NS: u32 = 350;
//const WS2812_RESET_US: u32 = 280;

#[derive(Debug)]
pub struct EspError {
    inner: esp_err_t,
}

impl std::fmt::Display for EspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "esp error code: {}", self.inner)
    }
}

impl std::error::Error for EspError {}

fn esp_res(err_code: esp_err_t) -> Result<(), EspError> {
    if err_code == ESP_OK as i32 {
        Ok(())
    } else {
        Err(EspError { inner: err_code })
    }
}

pub type Led = LedStrip<1>;

#[derive(Debug)]
pub struct LedStrip<const NUM_LEDS: usize> {
    ws2812_t0h_ticks: u32,
    ws2812_t0l_ticks: u32,
    ws2812_t1h_ticks: u32,
    ws2812_t1l_ticks: u32,
    buffer: [[u8; 3]; NUM_LEDS],
    channel: rmt_channel_t,
}

impl<const NUM_LEDS: usize> LedStrip<NUM_LEDS> {
    pub fn new(rmt_channel: rmt_channel_t, gpio_num: gpio_num_t) -> Result<Self, EspError> {
        let config = rmt_config_t {
            rmt_mode: rmt_mode_t_RMT_MODE_TX,
            gpio_num: gpio_num,
            channel: rmt_channel,
            clk_div: 2,
            mem_block_num: 1,
            flags: 0,
            __bindgen_anon_1: rmt_config_t__bindgen_ty_1 {
                tx_config: rmt_tx_config_t {
                    carrier_freq_hz: 38000,
                    carrier_level: rmt_carrier_level_t_RMT_CARRIER_LEVEL_HIGH,
                    idle_level: rmt_carrier_level_t_RMT_CARRIER_LEVEL_LOW,
                    carrier_duty_percent: 33,
                    carrier_en: false,
                    loop_en: false,
                    idle_output_en: true,
                    loop_count: 0,
                },
            },
        };

        let config_ptr: *const esp_idf_sys::rmt_config_t = &config;
        unsafe {
            esp_res(esp_idf_sys::rmt_config(config_ptr))?;
            esp_res(esp_idf_sys::rmt_driver_install(config.channel, 0, 0))?;
        }

        let mut counter_clk_hz: u32 = 0;
        unsafe {
            esp_res(esp_idf_sys::rmt_get_counter_clock(
                config.channel,
                &mut counter_clk_hz,
            ))?;
        }

        let ratio = counter_clk_hz as f32 / 1e9;

        let ws2812_t0h_ticks = (ratio * WS2812_T0H_NS as f32) as u32;
        let ws2812_t0l_ticks = (ratio * WS2812_T0L_NS as f32) as u32;
        let ws2812_t1h_ticks = (ratio * WS2812_T1H_NS as f32) as u32;
        let ws2812_t1l_ticks = (ratio * WS2812_T1L_NS as f32) as u32;

        unsafe {
            esp_res(esp_idf_sys::rmt_translator_init(
                config.channel,
                Some(Self::adapter),
            ))?;
        }

        let led_strip = LedStrip {
            ws2812_t0h_ticks,
            ws2812_t0l_ticks,
            ws2812_t1h_ticks,
            ws2812_t1l_ticks,
            buffer: [[0, 0, 0]; NUM_LEDS],
            channel: rmt_channel,
        };
        Ok(led_strip)
    }

    unsafe extern "C" fn adapter(
        src: *const c_void,
        dest: *mut esp_idf_sys::rmt_item32_t,
        src_size: u32,
        wanted_num: u32,
        translated_size: *mut u32,
        item_num: *mut u32,
    ) {
        if src.is_null() || dest.is_null() {
            *translated_size = 0;
            *item_num = 0;
            return;
        }
        let led_strip_p: *const Self = src.cast();
        let led_strip_ref: &Self = &*led_strip_p;
        info!("{:?}", led_strip_ref);
        let bit0 = Self::get_rmt_item32(
            led_strip_ref.ws2812_t0h_ticks,
            1,
            led_strip_ref.ws2812_t0l_ticks,
            0,
        );
        let bit1 = Self::get_rmt_item32(
            led_strip_ref.ws2812_t1h_ticks,
            1,
            led_strip_ref.ws2812_t1l_ticks,
            0,
        );
        let mut size = 0;
        let mut num = 0;
        let mut psrc: *const u8 = led_strip_ref.buffer.as_ptr().cast();
        let mut pdest = dest;
        while size < src_size && num < wanted_num {
            for i in 0..8 {
                // MSB first
                if *psrc & (1 << (7 - i)) != 0 {
                    (*pdest) = bit1;
                } else {
                    (*pdest) = bit0;
                }
                num += 1;
                pdest = pdest.add(1);
            }
            size += 1;
            psrc = psrc.add(1);
        }
        *translated_size = size;
        *item_num = num;
    }

    fn get_rmt_item32(
        duration0: u32,
        level0: u32,
        duration1: u32,
        level1: u32,
    ) -> esp_idf_sys::rmt_item32_t {
        let mut tmp = esp_idf_sys::rmt_item32_t__bindgen_ty_1__bindgen_ty_1::default();
        tmp.set_duration0(duration0);
        tmp.set_duration1(duration1);
        tmp.set_level0(level0);
        tmp.set_level1(level1);

        esp_idf_sys::rmt_item32_t {
            __bindgen_anon_1: esp_idf_sys::rmt_item32_t__bindgen_ty_1 {
                __bindgen_anon_1: tmp,
            },
        }
    }
}

impl<const NUM_LEDS: usize> Drop for LedStrip<NUM_LEDS> {
    fn drop(&mut self) {
        unsafe {
            esp_idf_sys::rmt_driver_uninstall(self.channel);
        }
    }
}

impl LedStrip<1> {
    pub fn set_color(&mut self, red: u8, green: u8, blue: u8) -> Result<(), EspError> {
        self.buffer[0] = [green, red, blue];
        self.update()
    }

    fn update(&mut self) -> Result<(), EspError> {
        unsafe {
            esp_res(esp_idf_sys::rmt_write_sample(
                self.channel,
                (self as *mut Self).cast(),
                1 as u32 * 3,
                true,
            ))?;
            esp_res(esp_idf_sys::rmt_wait_tx_done(self.channel, 1_000_000))?;
        }
        Ok(())
    }

    pub fn fade_to(
        &mut self,
        red: u8,
        green: u8,
        blue: u8,
        num_steps: u32,
        delay_per_step: Duration,
    ) -> Result<(), EspError> {
        let from_color = self.buffer[0];
        let to_color = [green, red, blue];

        for x in 1..=num_steps {
            for i in 0..3 {
                self.buffer[0][i] = ((to_color[i] as u32 * x
                    + from_color[i] as u32 * (num_steps - x))
                    / num_steps) as u8;
            }
            self.update()?;
            thread::sleep(delay_per_step);
        }

        Ok(())
    }
}
