pub mod board;
pub mod network;

/*
fn main() {
    //let device = Device::cuda_if_available(0).unwrap();
    let device = Device::Cpu;
    dbg!(Tensor::from_vec(vec![0u8, 1u8], (2,), &device).unwrap());

    let vb = VarBuilder::new_with_args(Box::new(VarMap::new()), DType::F32, &device);
    let mb = network::EfficientNet::new(vb).unwrap();

    let tensor = Tensor::ones((1, 12, 8, 8), candle_core::DType::F32, &device).unwrap();
    match mb.forward(&tensor) {
        Err(e) => println!("ERROR: {e}"),
        Ok((policy, value)) => {
            println!("{:?} = {}", policy.shape(), policy);
            println!("{:?} = {}", value.shape(), value);
        }
    }
}
    */
