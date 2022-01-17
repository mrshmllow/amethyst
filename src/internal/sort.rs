use crate::internal::{structs, rpc};

pub fn sort(a: &[String], verbosity: i32) -> structs::Sorted {
    #[allow(unused_mut)]
    let mut repo: Vec<String> = vec![];
    let mut aur: Vec<String> = vec![];
    let mut nf: Vec<String> = vec![];

    match verbosity {
        0 => {}
        1 => {
            eprintln!("Sorting:");
            eprintln!("{:?}", a);
        }
        _ => {
            eprintln!("Sorting:");
            for b in a {
                eprintln!("{:?}", b);
            }
        }
    }

    for b in a {
        if rpc::rpcinfo(b.to_string()).found {
            if verbosity >= 1 {
                eprintln!("{} found in AUR.", b);
            }
            aur.push(b.to_string());
        } else {
            if verbosity >= 1 {
                eprintln!("{} not found.", b);
            }
            nf.push(b.to_string());
        }
    }

    structs::Sorted::new(
        repo,
        aur,
        nf
    )
}