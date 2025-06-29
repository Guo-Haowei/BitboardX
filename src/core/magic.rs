use crate::core::types::{BitBoard, File, Rank, Square};

struct Magic {
    mask: BitBoard,
    magic: BitBoard,
    shift: u32,
    attack_table: Vec<BitBoard>,
}

/// Generate mask excluding edges on rank and file for rook
/// Don't include the square itself,
/// and don't include edges, because sliding pieces stops at edges anyway
/// e.g. for square D4, the mask will look like this:
/// a b c d e f g h
/// 0 0 0 0 0 0 0 0 | 8
/// 0 0 0 1 0 0 0 0 | 7
/// 0 0 0 1 0 0 0 0 | 6
/// 0 0 0 1 0 0 0 0 | 5
/// 0 1 1 0 1 1 1 0 | 4
/// 0 0 0 1 0 0 0 0 | 3
/// 0 0 0 1 0 0 0 0 | 2
/// 0 0 0 0 0 0 0 0 | 1
const fn relevant_blocker_mask_rook(square: Square) -> BitBoard {
    let mut mask = BitBoard::new();

    let (file, rank) = square.file_rank();

    let mut f = File::B.0;
    while f <= File::G.0 {
        if f != file.0 {
            mask.set(Square::make(File(f), rank).as_u8());
        }
        f += 1;
    }

    let mut r = Rank::_2.0;
    while r <= Rank::_7.0 {
        if r != rank.0 {
            mask.set(Square::make(file, Rank(r)).as_u8());
        }
        r += 1;
    }

    mask
}

fn generate_blocker_combination_helper(
    mask: u64,
    so_far: u64,
    depth: u8,
    combinations: &mut Vec<BitBoard>,
) {
    if depth == 0 {
        combinations.push(BitBoard::from(so_far));
        return;
    }

    let sq = mask.trailing_zeros() as u8;
    let sq_mask = 1u64 << sq;
    let mask = mask & (mask - 1);

    // either include the square or not
    generate_blocker_combination_helper(mask, so_far, depth - 1, combinations);
    generate_blocker_combination_helper(mask, so_far | sq_mask, depth - 1, combinations);
}

fn generate_blocker_combination(mask: BitBoard) -> Vec<BitBoard> {
    let count = mask.count() as u8;
    let combination_count = 1 << count; // 2^count combinations

    let mut combinations = Vec::with_capacity(combination_count);

    generate_blocker_combination_helper(mask.get(), 0, count, &mut combinations);

    debug_assert!(combinations.len() == combination_count);
    combinations
}

// fn sliding_attacks_rook(square: usize, blockers: BitBoard) -> BitBoard {
//     // Generate rook attacks on given blockers
// }

// fn find_magic_number(
//     square: usize,
//     relevant_bits: usize,
//     attacks_for_blockers: &Vec<(BitBoard, BitBoard)>,
// ) -> BitBoard {
//     // Try random candidates and check collisions for indexing
// }

// fn generate_magic_for_square(square: usize) -> Magic {
//     let mask = relevant_blocker_mask_rook(square);
//     let blocker_subsets = generate_blocker_subsets(mask);

//     // Precompute attacks for all blocker subsets
//     let attacks_for_blockers: Vec<(BitBoard, BitBoard)> = blocker_subsets
//         .iter()
//         .map(|&blockers| (*blockers, sliding_attacks_rook(square, blockers)))
//         .collect();

//     let relevant_bits = mask.count_ones() as usize;

//     let magic = find_magic_number(square, relevant_bits, &attacks_for_blockers);

//     let shift = 64 - relevant_bits as u32;

//     // Build attack table indexed by magic indexing
//     let mut attack_table = vec![0; 1 << relevant_bits];
//     for (blockers, attack) in &attacks_for_blockers {
//         let index = ((*blockers & mask).wrapping_mul(magic)) >> shift;
//         attack_table[index as usize] = *attack;
//     }

//     Magic { mask, magic, shift, attack_table }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_rook_relevant_blocker_mask() {
        let mask = relevant_blocker_mask_rook(Square::A1);

        assert_eq!(
            mask.to_string(),
            r#". . . . . . . .
X . . . . . . .
X . . . . . . .
X . . . . . . .
X . . . . . . .
X . . . . . . .
X . . . . . . .
. X X X X X X .
"#
        );

        let mask = relevant_blocker_mask_rook(Square::D4);
        assert_eq!(
            mask.to_string(),
            r#". . . . . . . .
. . . X . . . .
. . . X . . . .
. . . X . . . .
. X X . X X X .
. . . X . . . .
. . . X . . . .
. . . . . . . .
"#
        );
    }

    #[test]
    fn test_blocker_combination_generation() {
        let mask = relevant_blocker_mask_rook(Square::D4);
        let combinations = generate_blocker_combination(mask);

        let set: HashSet<u64> = combinations.iter().map(|item| item.get()).collect();
        assert_eq!(set.len(), combinations.len(), "Combinations should be unique");
    }

    // #[test]
    // fn test_magic_bitboard() {
    //     for sq in 0..64 {
    //         let magic = generate_magic_for_square(sq);
    //         // Save or print magic.mask, magic.magic, magic.shift, magic.attack_table
    //     }
    //     assert!(false);
    // }
}
