use sysproxy::Sysproxy;
// use sysproxy::Autoproxy;

fn main() -> Result<(), sysproxy::Error> {
    let mut sysproxy = Sysproxy::get_system_proxy().unwrap();
    // let autoproxy = Autoproxy::get_auto_proxy().unwrap();
    println!("The sysproxy is {:?}", sysproxy);
    // println!("The autoproxy is {:?}", autoproxy);

    sysproxy.host = "127.0.0.1".into();
    sysproxy.port = 7890;
    if sysproxy.enable {
        // Turn off
        sysproxy.enable = false;
        println!("Now we have turned off the proxy.")
    } else {
        // Turn on
        sysproxy.enable = true;
        println!("Now we have turned on the proxy.")
    }
    sysproxy.set_system_proxy()?;
    Ok(())
}
