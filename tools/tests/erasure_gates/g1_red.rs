fn a(rt: &Runtime) {
    rt.block_on(work()); //~
    futures::executor::block_on(work()); //~
    Handle::current().block_on(work()); //~
}
