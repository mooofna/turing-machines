// M*2: x^n becomes x^(2n), on one tape over {x, ␣}.
//
// TheoLog 2024, exercise sheet 1, task 4b; lecture 2 uses this machine
// to show that the busy beaver function is not computable. The machine
// removes the last x of the input and writes two x on the far side of a
// one-cell gap: one in the gap itself, one at the end. Every round walks
// over the whole output, so the machine needs time quadratic in n.
machine "M*2 (one tape)"
input "xxx"
output unary x

Q = {q0, q1, q2, q3, q4, q5}
Σ = {x}
Γ = {x, ␣}
start = q0
F = {q5}

δ(q0, x) = (q0, x, R)  // run to the end of the input
δ(q0, ␣) = (q1, ␣, L)

δ(q1, x) = (q2, ␣, R)  // remove the last x of the input
δ(q1, ␣) = (q5, ␣, R)  // the input is used up

δ(q2, ␣) = (q3, x, R)  // one x into the gap
δ(q3, x) = (q3, x, R)
δ(q3, ␣) = (q4, x, L)  // and one x at the end

δ(q4, x) = (q4, x, L)  // back across the output
δ(q4, ␣) = (q1, ␣, L)  // and across the gap
