use preinterpret::*;

fn main() {
    run! {
        let my_variable = "before";
        %[my_variable = "updated";].reinterpret_as_run();
    }
}
