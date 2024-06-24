use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{digit1, newline, space0, space1},
    combinator::{map, map_res, opt},
    multi::separated_list1,
    sequence::{preceded, terminated, tuple},
    IResult,
};

use crate::memory_map::{MemoryMap, Numeric, Range};

fn parse_size(input: &str) -> IResult<&str, Numeric> {
    map_res(terminated(digit1, tag(" kB")), |s: &str| {
        s.parse::<usize>().map(|num| Numeric::Kb(num))
    })(input)
}

fn parse_number(input: &str) -> IResult<&str, Numeric> {
    map_res(digit1, |s: &str| {
        s.parse::<usize>().map(|num| Numeric::Number(num))
    })(input)
}

fn parse_memory_line(input: &str) -> IResult<&str, (String, Numeric)> {
    map(
        tuple((
            take_until(":"),
            tag(":"),
            space1,
            alt((parse_size, parse_number)),
        )),
        |(label, _, _, value)| (label.to_string(), value),
    )(input)
}

fn parse_vm_flags(input: &str) -> IResult<&str, String> {
    map(
        tuple((tag("VmFlags:"), space0, take_while1(|c| c != '\n'))),
        |(_, _, flags): (&str, &str, &str)| flags.to_string(),
    )(input)
}

fn parse_hex(input: &str) -> IResult<&str, usize> {
    map_res(take_while1(|c: char| c.is_alphanumeric()), |num| {
        usize::from_str_radix(num, 16)
    })(input)
}

fn parse_memory_range(input: &str) -> IResult<&str, Range> {
    map(tuple((parse_hex, tag("-"), parse_hex)), |(from, _, to)| {
        Range { from, to }
    })(input)
}

pub fn parse_memory_map(input: &str) -> IResult<&str, MemoryMap> {
    let (input, (address_range, permissions, offset, device, inode, path, _)) = tuple((
        parse_memory_range,
        preceded(space1, take_while1(|c| c != ' ')),
        preceded(space1, take_while1(|c| c != ' ')),
        preceded(space1, take_while1(|c| c != ' ')),
        preceded(space1, take_while1(|c| c != ' ')),
        preceded(space1, take_while(|c| c != '\n')),
        tag("\n"),
    ))(input)?;

    let (input, sizes) = separated_list1(newline, parse_memory_line)(input)?;

    let (input, _) = tag("\n")(input)?;
    let sizes = sizes.into_iter().collect();
    let (input, vm_flags) = parse_vm_flags(input)?;

    let (input, _) = opt(tag("\n"))(input)?;

    Ok((
        input,
        MemoryMap {
            address_range,
            permissions: permissions.to_string(),
            offset: offset.to_string(),
            device: device.to_string(),
            inode: inode.to_string(),
            path: if path.is_empty() {
                None
            } else {
                Some(path.to_string())
            },
            sizes,
            vm_flags,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        assert_eq!(
            ("Size".to_string(), Numeric::Kb(4)),
            parse_memory_line("Size:                  4 kB\n")
                .unwrap()
                .1
        )
    }

    #[test]
    fn test_parse_memory_map() {
        let input = r#"6ff1475c000-56ff1475d000 r--p 00000000 fc:06 13134476                   /home/stephenwakely/src/c/usememory/a.out
Size:                  4 kB
KernelPageSize:        4 kB
MMUPageSize:           4 kB
Rss:                   4 kB
Pss:                   4 kB
Pss_Dirty:             0 kB
Shared_Clean:          0 kB
Shared_Dirty:          0 kB
Private_Clean:         4 kB
Private_Dirty:         0 kB
Referenced:            4 kB
Anonymous:             0 kB
LazyFree:              0 kB
AnonHugePages:         0 kB
ShmemPmdMapped:        0 kB
FilePmdMapped:         0 kB
Shared_Hugetlb:        0 kB
Private_Hugetlb:       0 kB
Swap:                  0 kB
SwapPss:               0 kB
Locked:                0 kB
THPeligible:    0
ProtectionKey:         0
VmFlags: rd mr mw me sd"#;

        let result = parse_memory_map(input);

        let sizes = [
            ("Size".to_string(), Numeric::Kb(4)),
            ("KernelPageSize".to_string(), Numeric::Kb(4)),
            ("MMUPageSize".to_string(), Numeric::Kb(4)),
            ("Rss".to_string(), Numeric::Kb(4)),
            ("Pss".to_string(), Numeric::Kb(4)),
            ("Pss_Dirty".to_string(), Numeric::Kb(0)),
            ("Shared_Clean".to_string(), Numeric::Kb(0)),
            ("Shared_Dirty".to_string(), Numeric::Kb(0)),
            ("Private_Clean".to_string(), Numeric::Kb(4)),
            ("Private_Dirty".to_string(), Numeric::Kb(0)),
            ("Referenced".to_string(), Numeric::Kb(4)),
            ("Anonymous".to_string(), Numeric::Kb(0)),
            ("LazyFree".to_string(), Numeric::Kb(0)),
            ("AnonHugePages".to_string(), Numeric::Kb(0)),
            ("ShmemPmdMapped".to_string(), Numeric::Kb(0)),
            ("FilePmdMapped".to_string(), Numeric::Kb(0)),
            ("Shared_Hugetlb".to_string(), Numeric::Kb(0)),
            ("Private_Hugetlb".to_string(), Numeric::Kb(0)),
            ("Swap".to_string(), Numeric::Kb(0)),
            ("SwapPss".to_string(), Numeric::Kb(0)),
            ("Locked".to_string(), Numeric::Kb(0)),
            ("THPeligible".to_string(), Numeric::Number(0)),
            ("ProtectionKey".to_string(), Numeric::Number(0)),
        ]
        .into_iter()
        .collect();

        let expected = MemoryMap {
            address_range: Range::try_from("6ff1475c000-56ff1475d000").unwrap(),
            permissions: "r--p".to_string(),
            offset: "00000000".to_string(),
            device: "fc:06".to_string(),
            inode: "13134476".to_string(),
            path: Some("/home/stephenwakely/src/c/usememory/a.out".to_string()),
            sizes,
            vm_flags: "rd mr mw me sd".to_string(),
        };

        assert_eq!(expected, result.unwrap().1);
    }

    #[test]
    fn test_another() {
        let input = r#"7a85b6dff000-7a85f6e00000 rw-p 00000000 00:00 0 
Size:            1048580 kB
KernelPageSize:        4 kB
MMUPageSize:           4 kB
Rss:                1028 kB
Pss:                1028 kB
Pss_Dirty:          1028 kB
Shared_Clean:          0 kB
Shared_Dirty:          0 kB
Private_Clean:         0 kB
Private_Dirty:      1028 kB
Referenced:         1028 kB
Anonymous:          1028 kB
LazyFree:              0 kB
AnonHugePages:         0 kB
ShmemPmdMapped:        0 kB
FilePmdMapped:         0 kB
Shared_Hugetlb:        0 kB
Private_Hugetlb:       0 kB
Swap:                  0 kB
SwapPss:               0 kB
Locked:                0 kB
THPeligible:    0
ProtectionKey:         0
VmFlags: rd wr mr mw me ac sd"#;

        let sizes = [
            ("Size".to_string(), Numeric::Kb(1048580)),
            ("KernelPageSize".to_string(), Numeric::Kb(4)),
            ("MMUPageSize".to_string(), Numeric::Kb(4)),
            ("Rss".to_string(), Numeric::Kb(1028)),
            ("Pss".to_string(), Numeric::Kb(1028)),
            ("Pss_Dirty".to_string(), Numeric::Kb(1028)),
            ("Shared_Clean".to_string(), Numeric::Kb(0)),
            ("Shared_Dirty".to_string(), Numeric::Kb(0)),
            ("Private_Clean".to_string(), Numeric::Kb(0)),
            ("Private_Dirty".to_string(), Numeric::Kb(1028)),
            ("Referenced".to_string(), Numeric::Kb(1028)),
            ("Anonymous".to_string(), Numeric::Kb(1028)),
            ("LazyFree".to_string(), Numeric::Kb(0)),
            ("AnonHugePages".to_string(), Numeric::Kb(0)),
            ("ShmemPmdMapped".to_string(), Numeric::Kb(0)),
            ("FilePmdMapped".to_string(), Numeric::Kb(0)),
            ("Shared_Hugetlb".to_string(), Numeric::Kb(0)),
            ("Private_Hugetlb".to_string(), Numeric::Kb(0)),
            ("Swap".to_string(), Numeric::Kb(0)),
            ("SwapPss".to_string(), Numeric::Kb(0)),
            ("Locked".to_string(), Numeric::Kb(0)),
            ("THPeligible".to_string(), Numeric::Number(0)),
            ("ProtectionKey".to_string(), Numeric::Number(0)),
        ]
        .into_iter()
        .collect();

        let expected = MemoryMap {
            address_range: Range::try_from("7a85b6dff000-7a85f6e00000").unwrap(),
            permissions: "rw-p".to_string(),
            path: None,
            offset: "00000000".to_string(),
            device: "00:00".to_string(),
            inode: "0".to_string(),
            sizes,
            vm_flags: "rd wr mr mw me ac sd".to_string(),
        };

        let result = parse_memory_map(input);

        assert_eq!(expected, result.unwrap().1);
    }
}
