extern crate timely;

use timely::dataflow::operators::*;
use timely::dataflow::channels::pact::Pipeline;

fn main() {
    timely::example(|scope| {

        let mut stash = ::std::collections::HashMap::new();

        // Feedback loop for noticing progress.
        let (handle, cycle) = scope.feedback(1);

        // Produce all numbers less than each input number.
        (1 .. 10u64)
            .to_stream(scope)
            // Assign timestamps to records so that not much work is in each time.
            .delay(|number, time| number / 2 )
            // Buffer records until all prior timestamps have completed.
            .binary_frontier(&cycle, Pipeline, Pipeline, "Buffer", move |capability, info| {

                let mut vector = Vec::new();

                move |input1, input2, output| {

                    // Stash received data.
                    input1.for_each(|time, data| {
                        data.swap(&mut vector);
                        stash.entry(time.retain())
                             .or_insert(Vec::new())
                             .extend(vector.drain(..));
                    });

                    // Consider sending stashed data.
                    for (time, data) in stash.iter_mut() {
                        // Only send data once the probe is not less than the time.
                        // That is, once we have finished all strictly prior work.
                        if !input2.frontier().less_than(time.time()) {
                            output.session(&time).give_iterator(data.drain(..));
                        }
                    }

                    // discard used capabilities.
                    stash.retain(|_time, data| !data.is_empty());
                }
            })
            .flat_map(|x| (0 .. x))
            // Discard data and connect back as an input.
            // .filter(|_| false)
            .connect_loop(handle);
    });
}
