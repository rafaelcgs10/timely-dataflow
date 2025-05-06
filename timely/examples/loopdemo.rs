extern crate timely;

use std::collections::HashMap;
use std::println;
use timely::dataflow::operators::*;

use timely::dataflow::operators::Inspect;
use timely::logging::TimelyEvent;
use timely::progress::reachability::logging::TrackerEvent;

use timely::Config;


fn main() {
    timely::execute(Config::thread(), |worker| {
        // let mut operators_addres: HashMap<usize, String> = HashMap::new();
        // let mut channels = HashMap::new();

        worker.log_register().insert::<TimelyEvent,_>("timely", print_graph());

        worker.log_register().insert::<TrackerEvent,_>("timely/reachability", |_time, data|
                                                       data.iter().for_each(|x| { match &x.2 {
                                                           TrackerEvent::SourceUpdate(su) => {
                                                               println!("Source updates: ");
                                                               su.updates.iter().for_each(|(ad, _, tp, count)| {
                                                                   print!("Location: {:?}, time: {:?}, delta: {:?}", ad, tp, count);
                                                                   println!("");
                                                               });
                                                               println!("");
                                                           },
                                                           TrackerEvent::TargetUpdate(tu) => println!("Target updates: {:?} in {:?}", tu.updates, tu.tracker_id),
                                                       }})
        );

        worker.dataflow::<usize,_,_>(|scope| {

            let (handle, stream) = scope.feedback(1);

        (1 .. 2)
            .to_stream(scope)
            .concat(&stream)
            .map(|x| if x % 2 == 0 { x / 2 } else { 3 * x + 1 } )
            .filter(|x| *x != 1)
            .branch_when(|t| t < &2).1
            .connect_loop(handle);
        });

    })
        .unwrap();
}

fn print_graph() -> impl FnMut(&std::time::Duration, &mut Vec<(std::time::Duration, usize, TimelyEvent)>) + 'static {
    let mut operators_name = HashMap::new();
    let mut operators_addres = HashMap::new();
    let mut channels = HashMap::new();
    let mut printed = false;

    move |_time, data: &mut Vec<(std::time::Duration, usize, TimelyEvent)>|
    {
        data.iter().for_each(|x| {
            match &x.2 {
                // TimelyEvent::PushProgress(pg) => println!("Push Progress: {:?}", pg),
                // TimelyEvent::GuardedProgress(pg) => println!("Guarded Progress: {:?}", pg),
                TimelyEvent::Operates(op) => { operators_name.insert(op.id, op.name.clone()); operators_addres.insert(op.id, op.addr.last().unwrap().clone()); },
                TimelyEvent::Channels(ch) => { channels.insert(ch.id, (ch.source.0, ch.target.0)); },
                TimelyEvent::CommChannels(_) => {
                    if !printed {
                        println!("_________");
                        println!("digraph graphname {{");
                        for (index, name) in operators_name.clone().into_iter() {
                            let index_string: &str = &index.to_string();
                            let mut name_string: String = name.to_owned();
                            name_string.push_str(" (");
                            name_string.push_str(index_string);
                            name_string.push_str(")");
                            if let Some(i) = operators_addres.get(&index) {
                                println!("{:} [label={:?}] ;", i, name_string);
                            };
                        };

                        for (index, (inn, out)) in channels.clone().into_iter() {
                            println!("  {:?} -> {:?} [label={:?}] ;", inn, out, index);
                        };

                        println!("}}");
                        println!("_________");
                        printed = true;
                    }
                }
                _ => ()
            }
        } );
    }
}
