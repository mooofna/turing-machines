// M*2 with two tapes: x^n becomes x^(2n) in linear time.
//
// TheoLog 2024, exercise sheet 1, task 4a. The machine copies the input
// to tape 2 while it reads it, then walks back over the copy on tape 2
// and writes one x on tape 1 for each x it finds.
//
// Without N moves both heads move on every step, so the sum of their
// positions stays even. Writing position n + i of tape 1 while reading
// position n - 1 - i of tape 2 would need the odd sum 2n - 1. So the copy
// leaves out the first x, the step into q2 writes the first new x, and
// then tape 1 writes position n + 1 + i while tape 2 reads n - 1 - i.
machine "M*2 (two tapes)"
tapes = 2
input "xxx"
output unary x

Q = {q0, q1, q2, q3}
Σ = {x}
Γ = {x, ␣}
start = q0
F = {q3}

δ(q0, (x, ␣)) = (q1, (x, ␣), (R, R))  // keep the first x out of the copy
δ(q0, (␣, ␣)) = (q3, (␣, ␣), (R, R))  // empty input: nothing to do

δ(q1, (x, ␣)) = (q1, (x, x), (R, R))  // copy the remaining n - 1
δ(q1, (␣, ␣)) = (q2, (x, ␣), (R, L))  // first new x; tape 2 turns back

δ(q2, (␣, x)) = (q2, (x, x), (R, L))  // one new x per copied x
δ(q2, (␣, ␣)) = (q3, (␣, ␣), (R, R))
