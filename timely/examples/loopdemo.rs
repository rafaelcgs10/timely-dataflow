extern crate timely;

use timely::dataflow::operators::*;

fn main() {
    timely::example(|scope| {

        // create a loop that cycles unboundedly.
        let (handle, stream) = scope.feedback(1);

        // circulate numbers, Collatz stepping each time.
        (1 .. 3)
            .to_stream(scope)
            .concat(&stream)
            .map(|x| if x % 2 == 0 { x / 2 } else { 3 * x + 1 } )
            .inspect(|x| println!("{:?}", x))
            .filter(|x| *x != 1)
            .branch_when(|t| t < &2).1
            .connect_loop(handle);
    });
}
