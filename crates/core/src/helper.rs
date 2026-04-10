pub fn get_rand_vid(min: u32, max: u32) -> u32 {
    rand::random_range(min..max)
}