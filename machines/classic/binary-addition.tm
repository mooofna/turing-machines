// Binary addition: u+v becomes the sum of u and v, both written with the
// most significant bit first.
//
// The machine removes the bits of v from the right, one per round, and
// adds each to the bit of u at the same position. Bits of u that have
// already received their partner are marked: O stands for 0 and I for 1.
// A carry runs left through the unmarked bits, and if u is shorter than
// v, new bits grow to the left of it. When v is used up, the machine
// erases the + and turns the marks back into digits.
machine "Binary addition"
input "101+11"
output binary msb

Q = {q0, q1, q2, q3, q4, q5, q6, q7, q8, q9}
Σ = {0, 1, +}
Γ = {0, 1, +, O, I, ␣}
start = q0
F = {q9}

δ(q0, {0, 1, +}) = (q0, *, R)  // run to the end of v
δ(q0, ␣) = (q1, ␣, L)

// Remove the last bit of v and carry it to the +.
δ(q1, 0) = (q2, ␣, L)
δ(q1, 1) = (q3, ␣, L)
δ(q1, +) = (q7, ␣, L)          // v is used up
δ(q2, {0, 1}) = (q2, *, L)
δ(q2, +) = (q4, +, L)
δ(q3, {0, 1}) = (q3, *, L)
δ(q3, +) = (q5, +, L)

// Add 0 to the next unmarked bit of u.
δ(q4, {O, I}) = (q4, *, L)
δ(q4, 0) = (q6, O, R)
δ(q4, 1) = (q6, I, R)
δ(q4, ␣) = (q6, O, R)          // u is shorter than v

// Add 1 to the next unmarked bit of u.
δ(q5, {O, I}) = (q5, *, L)
δ(q5, 0) = (q6, I, R)
δ(q5, 1) = (q8, O, L)          // 1 + 1 = 10: carry
δ(q5, ␣) = (q6, I, R)

// Propagate the carry.
δ(q8, 1) = (q8, 0, L)
δ(q8, {0, ␣}) = (q6, 1, R)

// Back to the end of v for the next bit.
δ(q6, {0, 1, O, I, +}) = (q6, *, R)
δ(q6, ␣) = (q1, ␣, L)

// Turn the marks back into digits.
δ(q7, O) = (q7, 0, L)
δ(q7, I) = (q7, 1, L)
δ(q7, {0, 1}) = (q7, *, L)
δ(q7, ␣) = (q9, ␣, R)
