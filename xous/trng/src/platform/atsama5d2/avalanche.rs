use atsama5d27::adc::{Adc, AdcChannel, StartupTime};

// Peripheral clock is around 80Mhz, max ADC clock is 20Mhz.
// The actual prescale divider is (this value + 1) * 2, so we
// are still well below the threshold.
const ADC_CLOCK_PRESCALER: u8 = 4;
const ADC_STARTUP_TIME: StartupTime = StartupTime::StartupTime24;
const NOISE_CHANNEL: AdcChannel = AdcChannel::Channel5;

pub(crate) struct AvalancheNoiseRng {
    adc: Adc,
}

impl AvalancheNoiseRng {
    pub(crate) fn new(adc: Adc) -> Self { Self { adc } }

    pub(crate) fn read_u32(&self) -> u32 {
        self.adc.reset();
        self.adc.set_prescaler(ADC_CLOCK_PRESCALER);
        self.adc.set_startup_time(ADC_STARTUP_TIME);
        self.adc.enable_channel(NOISE_CHANNEL);

        let result = self.read_noise_adc();

        self.adc.sleep();

        result
    }

    fn read_noise_adc(&self) -> u32 {
        let mut res = 0;
        for _ in 0..8 {
            res <<= 4;
            self.adc.start();
            let raw_noise = self.adc.read(NOISE_CHANNEL);
            // We get 12 bits of data, let's mix that together, so each output bit is mixed from 3 raw bits.
            // As long as at least one bit out of the 3 components are truly random, it doesn't matter if the
            // rest are not.
            let noise = raw_noise ^ (raw_noise >> 4) ^ (raw_noise >> 8);
            res ^= (noise & 15) as u32;
        }

        res
    }
}
