# Turing machines

This site combines a simulator for Turing machines with six chapters that lead from Turing's definition of the machine to open problems about the busy beaver function. The chapters contain interactive figures whose machines run in your browser: nothing is computed on a server, and the step counts and other results shown in the figures are produced by the simulator while you read. The project grew out of an exercise in the course Theoretische Informatik und Logik at TU Dresden [@theolog2024].

<<figure:hero | Space-time diagram of the five-state busy beaver champion, simulated in your browser when this page loaded; time runs downward and the tape runs horizontally. The machine halted after {{live:hero-steps}} steps with {{live:hero-ones}} ones on the tape, and the simulation took {{live:hero-time}}.>>

## Chapters

1. **The machine.** Turing's definition in the notation of the course, configurations and steps, and the multiplication machine from the exercise that started this project.
2. **Programs as data.** How a machine is written down as a word, why a single universal machine can run all others, and the text format used for small machines today.
3. **What cannot be computed.** The halting problem, the diagonal argument that shows it is undecidable, and Rice's theorem.
4. **The busy beaver.** Radó's functions $S$ and $\Sigma$, the proof that they are not computable, and the values that are known.
5. **Proving BB(5).** How the value for five states was determined and formally verified, with an enumeration of all small machines that runs in your browser.
6. **The frontier.** Six-state machines, Collatz-like problems such as Antihydra, and the point beyond which the axioms of set theory cannot determine the busy beaver function.

**Playground.** Write your own machines and run them in the simulator.

**Gallery.** Machines from the course exercises and from the busy beaver literature, each ready to run.
