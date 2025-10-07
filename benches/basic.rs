use preinterpret::benchmark_run;

macro_rules! benchmark {
    ($label:expr, { $($code:tt)* }) => {
        let output_str = benchmark_run!($($code)*);
        println!();
        println!("{:0>8}", $label);
        println!("{:0>8}", output_str);
    };
}

fn main() {
    benchmark!("Trivial Sum", { 1 + 1 + 1 });
    benchmark!("For loop adding up 1000 times", {
        let output = 0;
        for i in 1..=1000 {
            output += i
        }
        output
    });
    benchmark!("For loop concatenating to stream 1000 tokens", {
        let output = %[];
        for i in 1..=1000 {
            output += %[i];
        }
        output
    });
    benchmark!("Lots of casts", {
        0 as u32 as int as u8 as char as string as stream
    });
    benchmark!("Simple tuple impls", {
        for N in 0..=10 {
            let comma_separated_types = %[];
            for name in ('A'..).into_iter().take(N) {
                let ident = name.to_ident();
                comma_separated_types += %[#ident,];
            }
            %[
                impl<#comma_separated_types> MyTrait for (#comma_separated_types) {}
            ]
        }
    });
    benchmark!("Accessing single elements of a large array", {
        let array = [];
        for i in 0..1000 {
            array.push(i);
        }
        let sum = 0;
        for i in 0..100 {
            sum += array[i];
        }
    });
    benchmark!("Lazy iterator", {
        let last = 0;
        for i in 0..100000 {
            if i == 5 {
                last = i;
                break;
            }
        }
        %[].assert_eq(last, 5);
    });
}
