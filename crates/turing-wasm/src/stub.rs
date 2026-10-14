//! Placeholder implementations that return valid data of the right shape, so the front end can
//! be developed before the engines are connected. Replaced module by module.

use std::fmt::Write as _;

use crate::dto::{
    CommentsDto, DeciderCountDto, DeclaredDto, DecoderDto, DirDto, EdgeRouteDto, EdgeRoutesDto,
    EnumProgressDto, GraphEdgeDto, GraphLayoutDto, GraphNodeDto, KindDto, MachineSpecDto,
    PaletteDto, ParseResultDto, RecordDto, RunResultDto, RunStatusDto, SnapshotDto,
    SpaceTimeProgressDto, StateDto, StopReasonDto, TapeWindowDto, TransitionDto,
};

fn transition(from: &str, read: &str, to: &str, write: &str, dir: DirDto) -> TransitionDto {
    TransitionDto {
        from: from.into(),
        read: vec![read.into()],
        to: to.into(),
        write: vec![write.into()],
        dir: vec![dir],
        comment: None,
        leading: Vec::new(),
        span: None,
    }
}

pub(crate) fn sample_spec() -> MachineSpecDto {
    MachineSpecDto {
        name: "Binary increment".into(),
        kind: KindDto::Transducer,
        tapes: 1,
        blank: "␣".into(),
        input_alphabet: vec!["0".into(), "1".into()],
        tape_alphabet: vec!["␣".into(), "0".into(), "1".into()],
        states: vec![
            StateDto {
                name: "q0".into(),
                is_final: false,
            },
            StateDto {
                name: "q1".into(),
                is_final: true,
            },
        ],
        start: "q0".into(),
        transitions: vec![
            transition("q0", "1", "q0", "0", DirDto::R),
            transition("q0", "0", "q1", "1", DirDto::R),
            transition("q0", "␣", "q1", "1", DirDto::R),
        ],
        input: "1101".into(),
        output: DecoderDto::Word,
        declared: DeclaredDto::default(),
        comments: CommentsDto::default(),
    }
}

pub(crate) fn parse_source(source: &str) -> ParseResultDto {
    if source.trim().is_empty() {
        return ParseResultDto {
            spec: None,
            diagnostics: Vec::new(),
        };
    }
    ParseResultDto {
        spec: Some(sample_spec()),
        diagnostics: Vec::new(),
    }
}

pub(crate) fn from_standard(text: &str) -> ParseResultDto {
    let mut spec = sample_spec();
    spec.kind = KindDto::Beaver;
    spec.name = text.into();
    ParseResultDto {
        spec: Some(spec),
        diagnostics: Vec::new(),
    }
}

pub(crate) fn format_spec(spec: &MachineSpecDto, ascii: bool) -> String {
    let delta = if ascii { "delta" } else { "δ" };
    let mut out = String::new();
    if !spec.name.is_empty() {
        let _ = writeln!(out, "machine \"{}\"", spec.name);
    }
    for t in &spec.transitions {
        let dir: Vec<&str> = t
            .dir
            .iter()
            .map(|d| match d {
                DirDto::L => "L",
                DirDto::R => "R",
                DirDto::N => "N",
            })
            .collect();
        let _ = writeln!(
            out,
            "{delta}({}, {}) = ({}, {}, {})",
            t.from,
            t.read.join(", "),
            t.to,
            t.write.join(", "),
            dir.join(", ")
        );
    }
    out
}

pub(crate) fn graph_layout(spec: &MachineSpecDto, kind: &str) -> GraphLayoutDto {
    let _ = kind;
    let nodes: Vec<GraphNodeDto> = spec
        .states
        .iter()
        .enumerate()
        .map(|(i, s)| GraphNodeDto {
            name: s.name.clone(),
            is_start: s.name == spec.start,
            is_final: s.is_final,
            x: 60.0 + 140.0 * i as f64,
            y: 80.0,
        })
        .collect();
    let mut edges: Vec<GraphEdgeDto> = Vec::new();
    for (index, t) in spec.transitions.iter().enumerate() {
        let from = spec
            .states
            .iter()
            .position(|s| s.name == t.from)
            .unwrap_or(0);
        let to = spec.states.iter().position(|s| s.name == t.to).unwrap_or(0);
        let label = format!(
            "{} ↦ {}, {:?}",
            t.read.join(","),
            t.write.join(","),
            t.dir[0]
        );
        if let Some(e) = edges.iter_mut().find(|e| e.from == from && e.to == to) {
            e.labels.push(label);
            e.transitions.push(index);
        } else {
            edges.push(GraphEdgeDto {
                from,
                to,
                labels: vec![label],
                transitions: vec![index],
            });
        }
    }
    let width = 120.0 + 140.0 * spec.states.len().saturating_sub(1) as f64;
    GraphLayoutDto {
        nodes,
        edges,
        width,
        height: 200.0,
        radius: 22.0,
    }
}

pub(crate) fn graph_route(spec: &MachineSpecDto, xs: &[f64], ys: &[f64]) -> EdgeRoutesDto {
    let layout = graph_layout(spec, "auto");
    let routes = layout
        .edges
        .iter()
        .enumerate()
        .map(|(edge, e)| {
            let (x1, y1) = (
                xs.get(e.from).copied().unwrap_or(0.0),
                ys.get(e.from).copied().unwrap_or(0.0),
            );
            let (x2, y2) = (
                xs.get(e.to).copied().unwrap_or(0.0),
                ys.get(e.to).copied().unwrap_or(0.0),
            );
            let path = if e.from == e.to {
                format!("M {} {} c -20 -50 20 -50 10 0", x1 - 10.0, y1 - 20.0)
            } else {
                format!("M {x1} {y1} L {x2} {y2}")
            };
            EdgeRouteDto {
                edge,
                path,
                tip_x: x2,
                tip_y: y2,
                tip_angle: (y2 - y1).atan2(x2 - x1),
                label_x: f64::midpoint(x1, x2),
                label_y: f64::midpoint(y1, y2) - 30.0,
                label_anchor: "middle".into(),
            }
        })
        .collect();
    EdgeRoutesDto { routes }
}

pub(crate) struct Session {
    spec: MachineSpecDto,
    input: Vec<String>,
    pub(crate) step: f64,
}

impl Session {
    pub(crate) fn new(spec: MachineSpecDto, input: &str) -> Self {
        Self {
            spec,
            input: input.chars().map(String::from).collect(),
            step: 0.0,
        }
    }

    pub(crate) fn snapshot(&self, radius: u32) -> SnapshotDto {
        let len = self.input.len() as f64;
        let halted = self.step >= len;
        let head = self.step.min(len);
        let r = f64::from(radius);
        let start = head - r;
        let cells = (0..(2 * radius + 1))
            .map(|i| {
                let pos = start + f64::from(i);
                if pos >= 0.0 && pos < len {
                    self.input[pos as usize].clone()
                } else {
                    self.spec.blank.clone()
                }
            })
            .collect();
        SnapshotDto {
            step: head,
            state: if halted {
                "q1".into()
            } else {
                self.spec.start.clone()
            },
            status: if halted {
                RunStatusDto::Halted
            } else {
                RunStatusDto::Running
            },
            verdict: None,
            reason: None,
            tapes: vec![TapeWindowDto { head, start, cells }],
            extent: [0.0, head],
            configuration: format!("{} {}", self.spec.start, self.input.concat()),
            next_transition: (!halted).then_some(0),
            last_transition: (head > 0.0).then_some(0),
            ones: len,
            output: None,
            emitted: Vec::new(),
            emitted_count: 0.0,
        }
    }

    pub(crate) fn seek(&mut self, step: f64, radius: u32) -> SnapshotDto {
        self.step = step.clamp(0.0, self.input.len() as f64);
        self.snapshot(radius)
    }

    pub(crate) fn run(&mut self, max_steps: f64, radius: u32) -> RunResultDto {
        let before = self.step;
        let snapshot = self.seek(self.step + max_steps, radius);
        let reason = if snapshot.status == RunStatusDto::Halted {
            StopReasonDto::Halted
        } else {
            StopReasonDto::Budget
        };
        RunResultDto {
            snapshot,
            reason,
            steps_done: self.step - before,
        }
    }
}

pub(crate) struct SpaceTime {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl SpaceTime {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub(crate) fn advance(&self) -> SpaceTimeProgressDto {
        SpaceTimeProgressDto {
            rows_done: self.height,
            steps: f64::from(self.height),
            done: true,
            halted: true,
        }
    }

    pub(crate) fn rgba(&self, palette: &PaletteDto) -> Vec<u8> {
        let _ = palette;
        let mut out = Vec::with_capacity((self.width * self.height * 4) as usize);
        for y in 0..self.height {
            for x in 0..self.width {
                let ink = (x + y) % 17 == 0;
                let v = if ink { 40 } else { 255 };
                out.extend_from_slice(&[v, v, v, 255]);
            }
        }
        out
    }
}

pub(crate) fn enum_progress(nodes: f64, target: f64) -> EnumProgressDto {
    let done = nodes >= target;
    EnumProgressDto {
        nodes,
        halting: (nodes * 0.3).floor(),
        deciders: turing_bb::DeciderId::ALL
            .iter()
            .map(|d| DeciderCountDto {
                key: d.key().into(),
                label: d.label().into(),
                non_halting: 0.0,
            })
            .collect(),
        holdouts: 0.0,
        max_steps: done.then(|| RecordDto {
            value: 6.0,
            machine: "1RB1LB_1LA1RZ".into(),
        }),
        max_ones: done.then(|| RecordDto {
            value: 4.0,
            machine: "1RB1LB_1LA1RZ".into(),
        }),
        done,
    }
}
