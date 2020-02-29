
//! 16-bit Huffman compression and decompression.
//! Huffman compression and decompression routines written
//!	by Christian Rouet for his PIZ image file format.
// see https://github.com/AcademySoftwareFoundation/openexr/blob/88246d991e0318c043e6f584f7493da08a31f9f8/OpenEXR/IlmImf/ImfHuf.cpp

use std::io::{Read, Write, ErrorKind, Error};
use crate::error::IoResult;
use smallvec::alloc::collections::BinaryHeap;

// void
// hufUncompress (const char compressed[],
// 	       int nCompressed,
// 	       unsigned short raw[],
// 	       int nRaw)
// {
pub fn decompress(compressed: &[u8], result: &mut [u16]) -> IoResult<()> {
//     //
//     // need at least 20 bytes for header
//     //
//     if (nCompressed < 20 )
//     {
// 	if (nRaw != 0)
// 	    notEnoughData();
//
// 	return;
//     }
    if compressed.len() < 20 && !result.is_empty() {
        return Err(std::io::Error::new(ErrorKind::UnexpectedEof, "invalid huffman input"));
    }
//
//     int im = readUInt (compressed);
//     int iM = readUInt (compressed + 4);
//     // int tableLength = readUInt (compressed + 8);
//     int nBits = readUInt (compressed + 12);
    let mut remaining_bytes = compressed;

    let min_hcode_index = u32::read(remaining_bytes)?;
    let max_hcode_index = u32::read(remaining_bytes)?;
    let _table_len = u32::read(remaining_bytes)?;
    let bit_count = u32::read(remaining_bytes)?;
//
//     if (im < 0 || im >= HUF_ENCSIZE || iM < 0 || iM >= HUF_ENCSIZE)
// 	invalidTableSize();
    if min_hcode_index < 0 || min_hcode_index >= ENCODE_SIZE || max_hcode_index < 0 || max_hcode_index >= ENCODE_SIZE {
        return Err(Error::new(ErrorKind::InvalidData, "huffman table size"));
    }

//     TODO
//     const char *ptr = compressed + 20;
    let _padding = u32::read(remaining_bytes)?;

//
//     if ( ptr + (nBits+7 )/8 > compressed+nCompressed)
//     {
//         notEnoughData();
//         return;
//     }
    if compressed.len() < (bit_count + 7) / 8 {
        return Err(Error::new(ErrorKind::InvalidData, "huffman content length"));
    }

//     // Fast decoder needs at least 2x64-bits of compressed data, and
//     // needs to be run-able on this platform. Otherwise, fall back
//     // to the original decoder
//
//     if (FastHufDecoder::enabled() && nBits > 128) { // TODO
//         FastHufDecoder fhd (ptr, nCompressed - (ptr - compressed), im, iM, iM);
//         fhd.decode ((unsigned char*)ptr, nBits, raw, nRaw);
//     }
//     else {
//         AutoArray <Int64, HUF_ENCSIZE> freq;
//         AutoArray <HufDec, HUF_DECSIZE> hdec;
//         hufClearDecTable (hdec);
//
//         hufUnpackEncTable (&ptr,
//                            nCompressed - (ptr - compressed),
//                            im,
//                            iM,
//                            freq);

    let mut frequencies = [0_i64; ENCODE_SIZE];
    let h_decode = [Decode::default(); DECODE_SIZE];
    unpack_encoding_table(remaining_bytes, min_hcode_index, max_hcode_index, &mut frequencies);


//
//         try {
//             if (nBits > 8 * (nCompressed - (ptr - compressed)))
//                 invalidNBits();
//
//             hufBuildDecTable (freq, im, iM, hdec);
//             hufDecode (freq, hdec, ptr, nBits, iM, nRaw, raw);
//         }
//         catch (...) {
//             hufFreeDecTable (hdec);
//             throw;
//         }
//
//         hufFreeDecTable (hdec);
//     }
// }

    Ok(())
}

pub fn compress(_uncompressed: &[u16], _result: &mut [u8]) -> IoResult<()> {
    unimplemented!()
}



const ENCODE_BITS: usize = 16;			// literal (value) bit length
const DECODE_BITS: usize = 14;			// decoding bit size (>= 8)

const ENCODE_SIZE: usize = (1 << ENCODE_BITS) + 1;	// encoding table size
const DECODE_SIZE: usize =  1 << DECODE_BITS;	        // decoding table size
const DECODE_MASK: usize = DECODE_SIZE - 1;

const SHORT_ZEROCODE_RUN: i64 = 59;
const LONG_ZEROCODE_RUN: i64  = 63;
const SHORTEST_LONG_RUN: i64  = 2 + LONG_ZEROCODE_RUN - SHORT_ZEROCODE_RUN;
const LONGEST_LONG_RUN: i64   = 255 + SHORTEST_LONG_RUN;

//    struct HufDec
//    {				// short code		long code
//    //-------------------------------
//    int		len:8;		// code length		0
//    int		lit:24;		// lit			p size
//    int	*	p;		// 0			lits
//    };
struct Decode {
    len_8b: i8,             // short: code length   | long: 0
    lit_24b: i32,           // short: lit           | long: p size
    start_index: usize,     // short: 0,            | long: lits
}

// void
// hufUnpackEncTable
//     (const char**	pcode,		// io: ptr to packed table (updated)
//      int		ni,		// i : input size (in bytes)
//      int		im,		// i : min hcode index
//      int		iM,		// i : max hcode index
//      Int64*		hcode)		//  o: encoding table [HUF_ENCSIZE]

/// run-length-decompresses all zero runs from the packed table to the encoding table
fn unpack_encoding_table(packed: &mut &[u8], mut min_hcode_index: usize, max_hcode_index: usize, encoding_table: &mut [i64]) -> IoResult<()> {
//     const char *p = *pcode;
//     Int64 c = 0;
//     int lc = 0;
    let mut remaining_bytes = *packed;
    let mut c = 0;
    let mut lc = 0;

//
//     for (; im <= iM; im++)
//     {
    // for code_index in min_hcode_index ..= max_hcode_index {
    while min_hcode_index <= max_hcode_index {

// 	        if (p - *pcode > ni)
// 	            unexpectedEndOfTable();
        if remaining_bytes.len() < 1 { // TODO we do not need these errors as `read` handles those for us
            return Err(Error::new(ErrorKind::InvalidData, "huffman table length"));
        }
//
// 	        Int64 l = hcode[im] = getBits (6, c, lc, p); // code length
        let code_len = read_bits(6, &mut c, &mut lc, &mut remaining_bytes);
        encoding_table[code_index] = code_len;

//
// 	        if (l == (Int64) LONG_ZEROCODE_RUN)
// 	        {
        if code_len == LONG_ZEROCODE_RUN as i64 {
// 	            if (p - *pcode > ni)
// 		        unexpectedEndOfTable();
            if remaining_bytes.len() < 1 {
                return Err(Error::new(ErrorKind::InvalidData, "huffman table length"));
            }
//
// 	            int zerun = getBits (8, c, lc, p) + SHORTEST_LONG_RUN;
            let zerun = read_bits(8, &mut c, &mut lc, &mut remaining_bytes) + SHORTEST_LONG_RUN;
//
// 	            if (im + zerun > iM + 1) // TODO open new issue in openexr for negative length?
// 		            tableTooLong();
            if zerun < 0 || min_hcode_index as i64 + zerun > max_hcode_index as i64 + 1 {
                return Err(Error::new(ErrorKind::InvalidData, "huffman table length"));
            }
//
// 	            while (zerun--)
// 		            hcode[im++] = 0;
//
// 	            im--;
            for value in &mut encoding_table[min_hcode_index .. min_hcode_index + zerun as usize] {
                *value = 0;
            }

            min_hcode_index += zerun as usize; // TODO + or - 1

// 	        }
        }
// 	        else if (l >= (Int64) SHORT_ZEROCODE_RUN)
// 	        {
        else if code_len >= SHORT_ZEROCODE_RUN as i64 {
// 	            int zerun = l - SHORT_ZEROCODE_RUN + 2;
//
// 	            if (im + zerun > iM + 1)
// 		            tableTooLong();
//
// 	            while (zerun--)
// 		            hcode[im++] = 0;
//
// 	            im--;
// 	        }

            let zerun = code_len - SHORT_ZEROCODE_RUN + 2;
            if zerun < 0 || min_hcode_index as i64 + zerun > max_hcode_index as i64 + 1 {
                return Err(Error::new(ErrorKind::InvalidData, "huffman table length"));
            }

            for value in &mut encoding_table[min_hcode_index .. min_hcode_index + zerun as usize] {
                *value = 0;
            }

            min_hcode_index += zerun as usize; // TODO + or - 1
//     }
        }
        else {
            min_hcode_index += 1;
        }
    }
//
//     *pcode = const_cast<char *>(p);
    *packed = remaining_bytes;
//
//     hufCanonicalCodeTable (hcode);
    canonical_table(encoding_table);

    Ok(())
}


//    inline Int64
//    hufLength (Int64 code) code & 63;
fn length(code: i64) -> i64 { code & 63 }

//    inline Int64
//    hufCode (Int64 code) code >> 6;
fn code(code: i64) -> i64 { code >> 6 }

//    inline void
//    outputBits (int nBits, Int64 bits, Int64 &c, int &lc, char *&out)
//    {
//        c <<= nBits;
//        lc += nBits;
//
//        c |= bits;
//
//        while (lc >= 8)
//            *out++ = (c >> (lc -= 8));
//    }
fn write_bits(count: i64, bits: i64, c: &mut i64, lc: &mut i64, mut out: impl Write) {
    *c = *c << count;
    *lc += count;

    *c = *c | bits;

    while *lc >= 8 {
        *lc -= 8;
        out.write(&[ (*c >> *lc) as u8 ]).expect("bit write err"); // TODO make sure never or always wraps?
    }
}

//    inline Int64
//    getBits (int nBits, Int64 &c, int &lc, const char *&in)
//    {
//      while (lc < nBits)
//      {
//          c = (c << 8) | *(unsigned char *)(in++);
//          lc += 8;
//      }
//
//      lc -= nBits;
//      return (c >> lc) & ((1 << nBits) - 1);
//    }
// TODO replace this function with a `Reader` struct that remembers all the parameters??
fn read_bits(count: i64, c: &mut i64, lc: &mut i64, mut read: impl Read) -> i64 {
    while *lc < count {
        use crate::io::Data;
        *c = (*c << 8) | (u8::read(&mut read).expect("huffman read err") as i64);
        *lc += 8;
    }

    *lc -= count;

    (*c >> *lc) & ((1 << count) - 1)
}


// Build a "canonical" Huffman code table:
//	- for each (uncompressed) symbol, hcode contains the length
//	  of the corresponding code (in the compressed data)
//	- canonical codes are computed and stored in hcode
//	- the rules for constructing canonical codes are as follows:
//	  * shorter codes (if filled with zeroes to the right)
//	    have a numerically higher value than longer codes
//	  * for codes with the same length, numerical values
//	    increase with numerical symbol values
//	- because the canonical code table can be constructed from
//	  symbol lengths alone, the code table can be transmitted
//	  without sending the actual code values
//	- see http://www.compressconsult.com/huffman/

// hufCanonicalCodeTable (Int64 hcode[HUF_ENCSIZE])
fn canonical_table(h_code: &mut [i64]) {
    debug_assert_eq!(h_code.len(), ENCODE_SIZE);

    // Int64 n[59];
    // for (int i = 0; i <= 58; ++i)
    //    n[i] = 0;
    let mut n = [ 0_i64; 59 ];


    // For each i from 0 through 58, count the
    // number of different codes of length i, and
    // store the count in n[i].
    //
    //    for (int i = 0; i < HUF_ENCSIZE; ++i)
    //        n[hcode[i]] += 1;
    for &code in h_code.iter() {
        n[code as usize] += 1;
    }

    // For each i from 58 through 1, compute the
    // numerically lowest code with length i, and
    // store that code in n[i].
    //    Int64 c = 0;
    //
    //    for (int i = 58; i > 0; --i)
    //    {
    //        Int64 nc = ((c + n[i]) >> 1);
    //        n[i] = c;
    //        c = nc;
    //    }

    let mut c = 0_i64;
    for n in &mut n.iter_mut().rev() {
        let nc = (c + *n) >> 1;
        *n = c;
        c = nc;
    }

    // hcode[i] contains the length, l, of the
    // code for symbol i.  Assign the next available
    // code of length l to the symbol and store both
    // l and the code in hcode[i].

    //    for (int i = 0; i < HUF_ENCSIZE; ++i)
    //    {
    //        int l = hcode[i];
    //
    //        if (l > 0)
    //        hcode[i] = l | (n[l]++ << 6);
    //    }
    for code_i in h_code.iter_mut() {
        let l = *code_i;
        if l > 0 {
            *code_i = l | (n[l as usize] << 6);
            n[l as usize] += 1;
        }
    }
}


// Compute Huffman codes (based on frq input) and store them in frq:
//	- code structure is : [63:lsb - 6:msb] | [5-0: bit length];
//	- max code length is 58 bits;
//	- codes outside the range [im-iM] have a null length (unused values);
//	- original frequencies are destroyed;
//	- encoding tables are used by hufEncode() and hufBuildDecTable();
//
// NB: The following code "(*a == *b) && (a > b))" was added to ensure
//     elements in the heap with the same value are sorted by index.
//     This is to ensure, the STL make_heap()/pop_heap()/push_heap() methods
//     produced a resultant sorted heap that is identical across OSes.

//    struct FHeapCompare
//    {
//        bool operator () (Int64 *a, Int64 *b)
//    {
//    return ((*a > *b) || ((*a == *b) && (a > b)));
//    }
//    };
/*fn compare_heap(a: &i64, b: &i64) -> bool {
    (*a > *b) || ((*a == *b) && (a > b))
}*/


//    hufBuildEncTable
//        (Int64*	frq,	// io: input frequencies [HUF_ENCSIZE], output table
//         int*	im,	//  o: min frq index
//         int*	iM)	//  o: max frq index
//    {
fn build_encoding_table(
    frequencies: &mut [i64],  // input frequencies, output encoding table
) -> (usize, usize) // return frequency max min range
{
    debug_assert_eq!(frequencies.len(), ENCODE_SIZE);

    // This function assumes that when it is called, array frq
    // indicates the frequency of all possible symbols in the data
    // that are to be Huffman-encoded.  (frq[i] contains the number
    // of occurrences of symbol i in the data.)
    //
    // The loop below does three things:
    //
    // 1) Finds the minimum and maximum indices that point
    //    to non-zero entries in frq:
    //
    //     frq[im] != 0, and frq[i] == 0 for all i < im
    //     frq[iM] != 0, and frq[i] == 0 for all i > iM
    //
    // 2) Fills array fHeap with pointers to all non-zero
    //    entries in frq.
    //
    // 3) Initializes array hlink such that hlink[i] == i
    //    for all array entries.


    //    AutoArray <int, HUF_ENCSIZE> hlink;
    //    AutoArray <Int64 *, HUF_ENCSIZE> fHeap;
    let mut h_link = [0_i32; ENCODE_SIZE];
    let mut f_heap = [0_i64; ENCODE_SIZE];

    //    *im = 0;
    //
    //    while (!frq[*im])
    //        (*im)++;
    let min_frequency_index = {
        let mut index = 0;
        while frequencies[index] != 0 { index += 1; }
        index
    };

    //
    //    int nf = 0;
    //
    //    for (int i = *im; i < HUF_ENCSIZE; i++)
    //    {
    //        hlink[i] = i;
    //
    //        if (frq[i])
    //        {
    //            fHeap[nf] = &frq[i];
    //            nf++;
    //            *iM = i;
    //        }
    //    }
    let mut nf = 0;
    let mut max_frequency_index = 0;

    for index in 0 .. ENCODE_SIZE {
        h_link[index] = index as i32;

        if frequencies[index] != 0 {
            f_heap[nf] = index as i64; // &frequencies[index];
            nf += 1;
            max_frequency_index = index;
        }
    }

    // Add a pseudo-symbol, with a frequency count of 1, to frq;
    // adjust the fHeap and hlink array accordingly.  Function
    // hufEncode() uses the pseudo-symbol for run-length encoding.

    //    (*iM)++;
    //    frq[*iM] = 1;
    //    fHeap[nf] = &frq[*iM];
    //    nf++;
    max_frequency_index += 1;
    frequencies[max_frequency_index] = 1;
    f_heap[nf] = max_frequency_index as i64; // &frequencies[max_frequency_index];
    nf += 1;

    // Build an array, scode, such that scode[i] contains the number
    // of bits assigned to symbol i.  Conceptually this is done by
    // constructing a tree whose leaves are the symbols with non-zero
    // frequency:
    //
    //     Make a heap that contains all symbols with a non-zero frequency,
    //     with the least frequent symbol on top.
    //
    //     Repeat until only one symbol is left on the heap:
    //
    //         Take the two least frequent symbols off the top of the heap.
    //         Create a new node that has first two nodes as children, and
    //         whose frequency is the sum of the frequencies of the first
    //         two nodes.  Put the new node back into the heap.
    //
    // The last node left on the heap is the root of the tree.  For each
    // leaf node, the distance between the root and the leaf is the length
    // of the code for the corresponding symbol.
    //
    // The loop below doesn't actually build the tree; instead we compute
    // the distances of the leaves from the root on the fly.  When a new
    // node is added to the heap, then that node's descendants are linked
    // into a single linear list that starts at the new node, and the code
    // lengths of the descendants (that is, their distance from the root
    // of the tree) are incremented by one.

    //    make_heap (&fHeap[0], &fHeap[nf], FHeapCompare());
    let mut heap = BinaryHeap::from(f_heap.to_vec()); // TODO do not create vec in the first place?

    //    AutoArray <Int64, HUF_ENCSIZE> scode;
    //    memset (scode, 0, sizeof (Int64) * HUF_ENCSIZE);
    let mut s_code = [0_i64; ENCODE_SIZE ];

    //    while (nf > 1)
    //    {
    while nf > 1 {

        // Find the indices, mm and m, of the two smallest non-zero frq
        // values in fHeap, add the smallest frq to the second-smallest
        // frq, and remove the smallest frq value from fHeap.
        //
        //        int mm = fHeap[0] - frq;
        //        pop_heap (&fHeap[0], &fHeap[nf], FHeapCompare());
        //        --nf;
        let mm = heap.pop().expect("cannot pop heap bug");
        nf -= 1;

        //        int m = fHeap[0] - frq;
        //        pop_heap (&fHeap[0], &fHeap[nf], FHeapCompare());
        let m = heap.pop().expect("cannot pop heap bug");

        //        frq[m ] += frq[mm];
        //        push_heap (&fHeap[0], &fHeap[nf], FHeapCompare());
        frequencies[m] += frequencies[mm];
        heap.push(m); // m?????

        //        // The entries in scode are linked into lists with the
        //        // entries in hlink serving as "next" pointers and with
        //        // the end of a list marked by hlink[j] == j.
        //        //
        //        // Traverse the lists that start at scode[m] and scode[mm].
        //        // For each element visited, increment the length of the
        //        // corresponding code by one bit. (If we visit scode[j]
        //        // during the traversal, then the code for symbol j becomes
        //        // one bit longer.)
        //        //
        //        // Merge the lists that start at scode[m] and scode[mm]
        //        // into a single list that starts at scode[m].
        //
        //        // Add a bit to all codes in the first list.

        //        TODO
        //        for (int j = m; true; j = hlink[j]) {
        //            scode[j]++;
        //            assert (scode[j] <= 58);
        //
        //            if (hlink[j] == j) {
        //                // Merge the two lists.
        //
        //                hlink[j] = mm;
        //                break;
        //            }
        //        }
        let mut j = m;
        loop {
            s_code[j] += 1;
            assert!(s_code[j] <= 58);

            if h_link[j] == j {
                // merge the two lists
                h_link[j] = mm;
                break;
            }

            j = hlink[j];
        }

        //
        //        // Add a bit to all codes in the second list
        //        for (int j = mm; true; j = hlink[j]) {
        //            scode[j]++;
        //            assert (scode[j] <= 58);
        //
        //            if (hlink[j] == j)
        //              break;
        //        }
        //    }
        let mut j = mm;
        loop {
            s_code[j] += 1;
            assert!(s_code[j] <= 58);

            if h_link[j] == j {
                // merge the two lists
                h_link[j] = mm;
                break;
            }

            j = hlink[j];
        }

        // Build a canonical Huffman code table, replacing the code
        // lengths in scode with (code, code length) pairs.  Copy the
        // code table from scode into frq.

        //    hufCanonicalCodeTable (scode);
        //    memcpy (frq, scode, sizeof (Int64) * HUF_ENCSIZE);

        debug_assert_eq!(s_code.len(), ENCODE_SIZE);
        debug_assert_eq!(frequencies.len(), ENCODE_SIZE);

        canonical_table(&mut s_code);
        frequencies.copy_from_slice(&s_code);
    }

    (min_frequency_index, max_frequency_index)
}