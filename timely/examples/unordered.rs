extern crate timely;

use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::hash::Hash;
use std::println;
use timely::dataflow::channels::pact::Pipeline;
use timely::dataflow::operators::*;

use timely::dataflow::{ProbeHandle};
use timely::dataflow::operators::{Operator, Inspect, Probe};
use timely::logging::{TimelyEvent, TimelyProgressEvent};
use timely::progress::reachability::logging::{TrackerEvent, TrackerLogger};

use timely::Config;

fn main() {
    timely::execute(Config::thread(), |worker| {
        // let mut operators_addres: HashMap<usize, String> = HashMap::new();
        // let mut channels = HashMap::new();

        worker.log_register().insert::<TimelyEvent,_>("timely", print_graph());

        worker.log_register().insert::<TrackerEvent,_>("timely/reachability", |_time, data|
                                                       data.iter().for_each(|x| { match &x.2 {
                                                           TrackerEvent::SourceUpdate(su) => { println!("Source updates: "); su.updates.iter().for_each(|(ad, _, tp, count)| { print!("Location: {:?}, time: {:?}, count: {:?}", ad, tp, count); println!("");}); println!(""); },
                                                           TrackerEvent::TargetUpdate(tu) => println!("Target updates: {:?} in {:?}", tu.updates, tu.tracker_id),
                                                       }})
        );


        let mut probe = ProbeHandle::new();

        let (mut input, mut cap) = worker.dataflow::<usize,_,_>(|scope| {
            let (input, stream) = scope.new_unordered_input();
            stream
                .inspect_batch(move |t, xs| {
                    for x in xs.iter() {
                        println!("streamed {} @ {:?}", x, t)
                    }
                })
                .unary_frontier(Pipeline, "batcher", |_capability, _info| {
                    let mut buffer = HashMap::new();

                    move |input, output| {
                        while let Some((time, data)) = input.next() {
                            buffer
                                .entry(time.retain())
                                .or_insert(Vec::new())
                                .push(data.take());
                        }

                        for (key, val) in buffer.iter_mut() {
                            if !input.frontier().less_equal(key.time()) {
                                let mut session = output.session(key);
                                for mut batch in val.drain(..) {
                                    for value in batch.drain(..) {
                                        session.give(value);
                                    }
                                }
                            }
                        }

                        buffer.retain(|_key, val| !val.is_empty());
                    }
                })
                .inspect_batch(move |t, xs| {
                    for x in xs.iter() {
                        println!("batched {} @ {:?}", x, t)
                    }
                })
                .probe_with(&mut probe);

            input
        });

        cap = cap.delayed(&(1 as usize));

        input.session(cap.delayed(&2)).give(3);
        input.session(cap.delayed(&1)).give(0);
        input.session(cap.delayed(&5)).give(1);
        input.session(cap.delayed(&5)).give(2);

        worker.step();

        println!("Replaces initial cap by 4");
        cap = cap.delayed(&4);
        while probe.less_than(&4) {
            worker.step();
        }

        println!("Replaces cap 4 by 5");
        cap = cap.delayed(&5);
        while probe.less_than(&5) {
            worker.step();
        }

        println!("Replaces cap 5 by 7");
        cap = cap.delayed(&7);
        while probe.less_than(&7) {
            worker.step();
        }

        println!("Finish");

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
