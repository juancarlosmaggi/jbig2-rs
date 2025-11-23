use super::{HuffmanLine, HuffmanTable};
use crate::error::Jbig2Error;

pub fn get_standard_table(number: u32) -> Result<HuffmanTable, Jbig2Error> {
    if number == 0 || number > 15 {
        return Err(Jbig2Error::new("invalid standard Huffman table number"));
    }
    let lines = match number {
        1 => vec![
            HuffmanLine::new(vec![0, 1, 4, 0x0]),
            HuffmanLine::new(vec![16, 2, 8, 0x2]),
            HuffmanLine::new(vec![272, 3, 16, 0x6]),
            HuffmanLine::new(vec![65808, 3, 32, 0x7]), // upper
        ],
        2 => vec![
            HuffmanLine::new(vec![0, 1, 0, 0x0]),
            HuffmanLine::new(vec![1, 2, 0, 0x2]),
            HuffmanLine::new(vec![2, 3, 0, 0x6]),
            HuffmanLine::new(vec![3, 4, 3, 0xe]),
            HuffmanLine::new(vec![11, 5, 6, 0x1e]),
            HuffmanLine::new(vec![75, 6, 32, 0x3e]), // upper
            HuffmanLine::new(vec![6, 0x3f]),         // OOB
        ],
        3 => vec![
            HuffmanLine::new(vec![-256, 8, 8, 0xfe]),
            HuffmanLine::new(vec![0, 1, 0, 0x0]),
            HuffmanLine::new(vec![1, 2, 0, 0x2]),
            HuffmanLine::new(vec![2, 3, 0, 0x6]),
            HuffmanLine::new(vec![3, 4, 3, 0xe]),
            HuffmanLine::new(vec![11, 5, 6, 0x1e]),
            HuffmanLine::new(vec![-257, 8, 32, 0xff, 1]), // lower
            HuffmanLine::new(vec![75, 7, 32, 0x7e]),      // upper
            HuffmanLine::new(vec![6, 0x3e]),              // OOB
        ],
        4 => vec![
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 2, 0, 0x2]),
            HuffmanLine::new(vec![3, 3, 0, 0x6]),
            HuffmanLine::new(vec![4, 4, 3, 0xe]),
            HuffmanLine::new(vec![12, 5, 6, 0x1e]),
            HuffmanLine::new(vec![76, 5, 32, 0x1f]), // upper
        ],
        5 => vec![
            HuffmanLine::new(vec![-255, 7, 8, 0x7e]),
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 2, 0, 0x2]),
            HuffmanLine::new(vec![3, 3, 0, 0x6]),
            HuffmanLine::new(vec![4, 4, 3, 0xe]),
            HuffmanLine::new(vec![12, 5, 6, 0x1e]),
            HuffmanLine::new(vec![-256, 7, 32, 0x7f, 1]), // lower
            HuffmanLine::new(vec![76, 6, 32, 0x3e]),      // upper
        ],
        6 => vec![
            HuffmanLine::new(vec![-2048, 5, 10, 0x1c]),
            HuffmanLine::new(vec![-1024, 4, 9, 0x8]),
            HuffmanLine::new(vec![-512, 4, 8, 0x9]),
            HuffmanLine::new(vec![-256, 4, 7, 0xa]),
            HuffmanLine::new(vec![-128, 5, 6, 0x1d]),
            HuffmanLine::new(vec![-64, 5, 5, 0x1e]),
            HuffmanLine::new(vec![-32, 4, 5, 0xb]),
            HuffmanLine::new(vec![0, 2, 7, 0x0]),
            HuffmanLine::new(vec![128, 3, 7, 0x2]),
            HuffmanLine::new(vec![256, 3, 8, 0x3]),
            HuffmanLine::new(vec![512, 4, 9, 0xc]),
            HuffmanLine::new(vec![1024, 4, 10, 0xd]),
            HuffmanLine::new(vec![-2049, 6, 32, 0x3e, 1]), // lower
            HuffmanLine::new(vec![2048, 6, 32, 0x3f]),     // upper
        ],
        7 => vec![
            HuffmanLine::new(vec![-1024, 4, 9, 0x8]),
            HuffmanLine::new(vec![-512, 3, 8, 0x0]),
            HuffmanLine::new(vec![-256, 4, 7, 0x9]),
            HuffmanLine::new(vec![-128, 5, 6, 0x1a]),
            HuffmanLine::new(vec![-64, 5, 5, 0x1b]),
            HuffmanLine::new(vec![-32, 4, 5, 0xa]),
            HuffmanLine::new(vec![0, 4, 5, 0xb]),
            HuffmanLine::new(vec![32, 5, 5, 0x1c]),
            HuffmanLine::new(vec![64, 5, 6, 0x1d]),
            HuffmanLine::new(vec![128, 4, 7, 0xc]),
            HuffmanLine::new(vec![256, 3, 8, 0x1]),
            HuffmanLine::new(vec![512, 3, 9, 0x2]),
            HuffmanLine::new(vec![1024, 3, 10, 0x3]),
            HuffmanLine::new(vec![-1025, 5, 32, 0x1e, 1]), // lower
            HuffmanLine::new(vec![2048, 5, 32, 0x1f]),     // upper
        ],
        8 => vec![
            HuffmanLine::new(vec![-15, 8, 3, 0xfc]),
            HuffmanLine::new(vec![-7, 9, 1, 0x1fc]),
            HuffmanLine::new(vec![-5, 8, 1, 0xfd]),
            HuffmanLine::new(vec![-3, 9, 0, 0x1fd]),
            HuffmanLine::new(vec![-2, 7, 0, 0x7c]),
            HuffmanLine::new(vec![-1, 4, 0, 0xa]),
            HuffmanLine::new(vec![0, 2, 1, 0x0]),
            HuffmanLine::new(vec![2, 5, 0, 0x1a]),
            HuffmanLine::new(vec![3, 6, 0, 0x3a]),
            HuffmanLine::new(vec![4, 3, 4, 0x4]),
            HuffmanLine::new(vec![20, 6, 1, 0x3b]),
            HuffmanLine::new(vec![22, 4, 4, 0xb]),
            HuffmanLine::new(vec![38, 4, 5, 0xc]),
            HuffmanLine::new(vec![70, 5, 6, 0x1b]),
            HuffmanLine::new(vec![134, 5, 7, 0x1c]),
            HuffmanLine::new(vec![262, 6, 7, 0x3c]),
            HuffmanLine::new(vec![390, 7, 8, 0x7d]),
            HuffmanLine::new(vec![646, 6, 10, 0x3d]),
            HuffmanLine::new(vec![-16, 9, 32, 0x1fe, 1]), // lower
            HuffmanLine::new(vec![1670, 9, 32, 0x1ff]),   // upper
            HuffmanLine::new(vec![2, 0x1]),               // OOB
        ],
        9 => vec![
            HuffmanLine::new(vec![-31, 8, 4, 0xfc]),
            HuffmanLine::new(vec![-15, 9, 2, 0x1fc]),
            HuffmanLine::new(vec![-11, 8, 2, 0xfd]),
            HuffmanLine::new(vec![-7, 9, 1, 0x1fd]),
            HuffmanLine::new(vec![-5, 7, 1, 0x7c]),
            HuffmanLine::new(vec![-3, 4, 1, 0xa]),
            HuffmanLine::new(vec![-1, 3, 1, 0x2]),
            HuffmanLine::new(vec![1, 3, 1, 0x3]),
            HuffmanLine::new(vec![3, 5, 1, 0x1a]),
            HuffmanLine::new(vec![5, 6, 1, 0x3a]),
            HuffmanLine::new(vec![7, 3, 5, 0x4]),
            HuffmanLine::new(vec![39, 6, 2, 0x3b]),
            HuffmanLine::new(vec![43, 4, 5, 0xb]),
            HuffmanLine::new(vec![75, 4, 6, 0xc]),
            HuffmanLine::new(vec![139, 5, 7, 0x1b]),
            HuffmanLine::new(vec![267, 5, 8, 0x1c]),
            HuffmanLine::new(vec![523, 6, 8, 0x3c]),
            HuffmanLine::new(vec![779, 7, 9, 0x7d]),
            HuffmanLine::new(vec![1291, 6, 11, 0x3d]),
            HuffmanLine::new(vec![-32, 9, 32, 0x1fe, 1]), // lower
            HuffmanLine::new(vec![3339, 9, 32, 0x1ff]),   // upper
            HuffmanLine::new(vec![2, 0x0]),               // OOB
        ],
        10 => vec![
            HuffmanLine::new(vec![-21, 7, 4, 0x7a]),
            HuffmanLine::new(vec![-5, 8, 0, 0xfc]),
            HuffmanLine::new(vec![-4, 7, 0, 0x7b]),
            HuffmanLine::new(vec![-3, 5, 0, 0x18]),
            HuffmanLine::new(vec![-2, 2, 2, 0x0]),
            HuffmanLine::new(vec![2, 5, 0, 0x19]),
            HuffmanLine::new(vec![3, 6, 0, 0x36]),
            HuffmanLine::new(vec![4, 7, 0, 0x7c]),
            HuffmanLine::new(vec![5, 8, 0, 0xfd]),
            HuffmanLine::new(vec![6, 2, 6, 0x1]),
            HuffmanLine::new(vec![70, 5, 5, 0x1a]),
            HuffmanLine::new(vec![102, 6, 5, 0x37]),
            HuffmanLine::new(vec![134, 6, 6, 0x38]),
            HuffmanLine::new(vec![198, 6, 7, 0x39]),
            HuffmanLine::new(vec![326, 6, 8, 0x3a]),
            HuffmanLine::new(vec![582, 6, 9, 0x3b]),
            HuffmanLine::new(vec![1094, 6, 10, 0x3c]),
            HuffmanLine::new(vec![2118, 7, 11, 0x7d]),
            HuffmanLine::new(vec![-22, 8, 32, 0xfe, 1]), // lower
            HuffmanLine::new(vec![4166, 8, 32, 0xff]),   // upper
            HuffmanLine::new(vec![2, 0x2]),              // OOB
        ],
        11 => vec![
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 2, 1, 0x2]),
            HuffmanLine::new(vec![4, 4, 0, 0xc]),
            HuffmanLine::new(vec![5, 4, 1, 0xd]),
            HuffmanLine::new(vec![7, 5, 1, 0x1c]),
            HuffmanLine::new(vec![9, 5, 2, 0x1d]),
            HuffmanLine::new(vec![13, 6, 2, 0x3c]),
            HuffmanLine::new(vec![17, 7, 2, 0x7a]),
            HuffmanLine::new(vec![21, 7, 3, 0x7b]),
            HuffmanLine::new(vec![29, 7, 4, 0x7c]),
            HuffmanLine::new(vec![45, 7, 5, 0x7d]),
            HuffmanLine::new(vec![77, 7, 6, 0x7e]),
            HuffmanLine::new(vec![141, 7, 32, 0x7f]), // upper
        ],
        12 => vec![
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 2, 0, 0x2]),
            HuffmanLine::new(vec![3, 3, 1, 0x6]),
            HuffmanLine::new(vec![5, 5, 0, 0x1c]),
            HuffmanLine::new(vec![6, 5, 1, 0x1d]),
            HuffmanLine::new(vec![8, 6, 1, 0x3c]),
            HuffmanLine::new(vec![10, 7, 0, 0x7a]),
            HuffmanLine::new(vec![11, 7, 1, 0x7b]),
            HuffmanLine::new(vec![13, 7, 2, 0x7c]),
            HuffmanLine::new(vec![17, 7, 3, 0x7d]),
            HuffmanLine::new(vec![25, 7, 4, 0x7e]),
            HuffmanLine::new(vec![41, 8, 5, 0xfe]),
            HuffmanLine::new(vec![73, 8, 32, 0xff]), // upper
        ],
        13 => vec![
            HuffmanLine::new(vec![1, 1, 0, 0x0]),
            HuffmanLine::new(vec![2, 3, 0, 0x4]),
            HuffmanLine::new(vec![3, 4, 0, 0xc]),
            HuffmanLine::new(vec![4, 5, 0, 0x1c]),
            HuffmanLine::new(vec![5, 4, 1, 0xd]),
            HuffmanLine::new(vec![7, 3, 3, 0x5]),
            HuffmanLine::new(vec![15, 6, 1, 0x3a]),
            HuffmanLine::new(vec![17, 6, 2, 0x3b]),
            HuffmanLine::new(vec![21, 6, 3, 0x3c]),
            HuffmanLine::new(vec![29, 6, 4, 0x3d]),
            HuffmanLine::new(vec![45, 6, 5, 0x3e]),
            HuffmanLine::new(vec![77, 7, 6, 0x7e]),
            HuffmanLine::new(vec![141, 7, 32, 0x7f]), // upper
        ],
        14 => vec![
            HuffmanLine::new(vec![-2, 3, 0, 0x4]),
            HuffmanLine::new(vec![-1, 3, 0, 0x5]),
            HuffmanLine::new(vec![0, 1, 0, 0x0]),
            HuffmanLine::new(vec![1, 3, 0, 0x6]),
            HuffmanLine::new(vec![2, 3, 0, 0x7]),
        ],
        15 => vec![
            HuffmanLine::new(vec![-24, 7, 4, 0x7c]),
            HuffmanLine::new(vec![-8, 6, 2, 0x3c]),
            HuffmanLine::new(vec![-4, 5, 1, 0x1c]),
            HuffmanLine::new(vec![-2, 4, 0, 0xc]),
            HuffmanLine::new(vec![-1, 3, 0, 0x4]),
            HuffmanLine::new(vec![0, 1, 0, 0x0]),
            HuffmanLine::new(vec![1, 3, 0, 0x5]),
            HuffmanLine::new(vec![2, 4, 0, 0xd]),
            HuffmanLine::new(vec![3, 5, 1, 0x1d]),
            HuffmanLine::new(vec![5, 6, 2, 0x3d]),
            HuffmanLine::new(vec![9, 7, 4, 0x7d]),
            HuffmanLine::new(vec![-25, 7, 32, 0x7e, 1]), // lower
            HuffmanLine::new(vec![25, 7, 32, 0x7f]),     // upper
        ],
        _ => {
            return Err(Jbig2Error::new(&format!(
                "standard table B.{} does not exist",
                number
            )));
        }
    };
    Ok(HuffmanTable::new(lines, true))
}
