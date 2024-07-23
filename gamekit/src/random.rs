//!
//! Randomm
//!

use rand::Rng;


//static mut RANDOM_GENERATOR: rand::rngs::ThreadRng = rand::thread_rng();

struct RandomContext {
    pub rng: Option<rand::rngs::ThreadRng>
}

static mut RANDOM: RandomContext = RandomContext {
    rng: None
};

pub struct Random {

}

impl Random {

    pub fn get_float() -> f32 {

        #[allow(static_mut_refs)]
        let value = unsafe { match &mut RANDOM.rng {
            Some(rng) => { rng.gen::<f32>() },
            None => {
                let mut rng = rand::thread_rng();
                let value = rng.gen::<f32>();
                RANDOM.rng = Some(rng);
                value
            }
        } };

        value
    }

    pub fn get_float_range(range_min: f32, range_max: f32) -> f32 {

        if range_max <= range_min {
            return range_min;
        }

        let range = range_max - range_min;
        let value = Self::get_float();
        let value_in_range = range_min + value * range;

        value_in_range
    }

}
