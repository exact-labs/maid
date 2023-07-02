use global_placeholders::init;

pub fn init() {
    init!("maid.cache_dir", ".maid/cache/{}/target");
}
