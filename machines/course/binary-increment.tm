// Binary increment, least significant bit first.
//
// The first machine of the 2024 simulator: it adds one to a binary number
// whose least significant bit stands leftmost, so the carry travels right.
// For example 1101 (eleven) becomes 0011 (twelve).
machine "Binary increment"
input "1101"
output binary lsb

Q = {q0, q1}
Σ = {0, 1}
Γ = {0, 1, ␣}
start = q0
F = {q1}

δ(q0, 1) = (q0, 0, R)  // 1 + 1 = 10: write 0 and carry on
δ(q0, 0) = (q1, 1, R)  // the carry stops here
δ(q0, ␣) = (q1, 1, R)  // the carry becomes a new most significant bit
