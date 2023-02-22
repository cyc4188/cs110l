use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    output_vec.resize_with(input_vec.len(), Default::default);
    let (in_sender, in_receiver) = crossbeam_channel::unbounded();
    let (out_sender, out_receiver) = crossbeam_channel::unbounded();

    let mut threads = Vec::new();
    for _ in 0..num_threads {
        let in_receiver = in_receiver.clone();
        let out_sender = out_sender.clone();
        let thread = thread::spawn(move || {
            while let Ok(input) = in_receiver.recv() {
                let (idx, input) = input;
                let output = f(input);
                out_sender.send((idx, output)).unwrap();
            }
        });
        threads.push(thread);
    }

    for (idx, input) in input_vec.drain(..).enumerate() {
        in_sender.send((idx, input)).unwrap();
    }

    drop(in_sender);
    drop(out_sender);

    while let Ok((idx, output)) = out_receiver.recv() {
        output_vec[idx] = output;
    }


    for thread in threads {
        thread.join().unwrap();
    }

    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
