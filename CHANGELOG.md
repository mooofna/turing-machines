# Changelog

## [1.1.0](https://github.com/mooofna/turing-machines/compare/v1.0.0...v1.1.0) (2026-10-15)


### Features

* **web:** add Open Graph metadata and a preview image for link previews ([3ee7199](https://github.com/mooofna/turing-machines/commit/3ee71999705692f357928b54c32f3ee14716d5dc))

## 1.0.0 (2026-10-15)


### Features

* **bb:** check Antihydra's rules on the machine and walk the rule ([5354503](https://github.com/mooofna/turing-machines/commit/53545031b42493ddc3e1e4784ae9a3883d4b0ac7))
* **bb:** compute up-arrow towers and both machine counts ([eac7ad9](https://github.com/mooofna/turing-machines/commit/eac7ad9a46f879fef3c24dc9ecd0081966e5f2f7))
* **bb:** enumerate the tree normal form in Coq-BB5 order ([f51f7dd](https://github.com/mooofna/turing-machines/commit/f51f7dd2b0e71d1b688a7466760df3a4ce662dab))
* **bb:** pack compact machines into one byte per transition ([3d92692](https://github.com/mooofna/turing-machines/commit/3d92692ed7df61bd9b8ff4d35e71403f6e5ea268))
* **bb:** port the Loops, n-gram CPS and RepWL deciders of Coq-BB5 ([f7faf91](https://github.com/mooofna/turing-machines/commit/f7faf912fd3266ed5caeb677d13474b1fae2cdff))
* **cli:** add the tm command with run, trace, check, fmt, convert, enumerate, decide, render and antihydra ([e811b04](https://github.com/mooofna/turing-machines/commit/e811b0437ea49655edc4b2d565e346103068343f))
* **cli:** polish the Antihydra report and show it in the README ([b9000f2](https://github.com/mooofna/turing-machines/commit/b9000f26545c75eec41c169da80d447d3a938610))
* **cli:** print through proved loops of enumerators and add end-to-end test scaffolding ([c80e7f3](https://github.com/mooofna/turing-machines/commit/c80e7f364da9fa601d1d9d071c058a4a7e5627c7))
* **cli:** show the machine shape on its own line and merge duplicate diagnostics ([a166eed](https://github.com/mooofna/turing-machines/commit/a166eed94f2376f6892c491335169fa514b221cd))
* **cli:** simulate undecided machines in tm decide and polish the enumeration funnel ([a1f6be5](https://github.com/mooofna/turing-machines/commit/a1f6be5646771300e304ba40079599353a8939ce))
* **core:** decode transducer output as word, unary or binary number ([ec696c6](https://github.com/mooofna/turing-machines/commit/ec696c604a1f9cc883c073528817df2e25644654))
* **core:** format machine specs canonically ([9742c5e](https://github.com/mooofna/turing-machines/commit/9742c5e690f21993a8a29a3d1b6987c93df6c6ee))
* **core:** implement exact seeking with checkpoint timelines ([bf6aa73](https://github.com/mooofna/turing-machines/commit/bf6aa73ea8f03577d0c35ff38298a04785256f35))
* **core:** implement the chain simulator on a run-length tape ([bfaa04d](https://github.com/mooofna/turing-machines/commit/bfaa04d2e3c5198e4632c733eb0082aa884307d8))
* **core:** implement the dense two-way infinite tape ([65ef75d](https://github.com/mooofna/turing-machines/commit/65ef75d749208dcce5b570144173fb7300249a5d))
* **core:** implement the reference simulator for k tapes ([f026b15](https://github.com/mooofna/turing-machines/commit/f026b15b4de60843bac20eae3be9617a55c1a357))
* **core:** infer, check and compile machine specs ([1d0bab4](https://github.com/mooofna/turing-machines/commit/1d0bab4dcc2f71a56af5fb784802e53fb5fa991e))
* **core:** let both simulators run on through proved loops ([03c6950](https://github.com/mooofna/turing-machines/commit/03c695067b506d451a006d14415e8135329a88f0))
* **core:** parse the description language with recovery per line ([968ab7c](https://github.com/mooofna/turing-machines/commit/968ab7c0db7125f188657070d5b72a615185f62e))
* **core:** read and write the bbchallenge standard format ([123dc7d](https://github.com/mooofna/turing-machines/commit/123dc7dd00e9df23f2e113d12af1d036720439c9))
* **core:** tokenize the description language ([a6df31e](https://github.com/mooofna/turing-machines/commit/a6df31e1e971e5c7600f8b0fb86f0cfc84ad37a7))
* **core:** validate and build machines with MachineBuilder ([c1149ae](https://github.com/mooofna/turing-machines/commit/c1149aeb9b8295a9b13e57497ffb102d973d7175))
* **core:** write configurations in course notation u q v ([d551e9c](https://github.com/mooofna/turing-machines/commit/d551e9cd5e313481748013675c15eb5b3de96608))
* **machines:** add the course and classic machines and the gallery ([3abb6e3](https://github.com/mooofna/turing-machines/commit/3abb6e3f20a9295a2bf749411d32b81f552d8c62))
* **machines:** open the playground with a commented increment machine ([8e25ed4](https://github.com/mooofna/turing-machines/commit/8e25ed435cd04d3f8076f69265e5d399e931ba65))
* **playground:** editor, simulator controls and share links ([7957e9c](https://github.com/mooofna/turing-machines/commit/7957e9caf0dccafca8db0103ff9a98b452cc7c0b))
* **playground:** open gallery machines and draw the space-time diagram ([94b029e](https://github.com/mooofna/turing-machines/commit/94b029e174ca1069a6bef2a160c8dcde34f513ad))
* **render:** lay out, route and draw state diagrams ([59fc476](https://github.com/mooofna/turing-machines/commit/59fc476bfd35e93b2399828134b7afffa3d54d59))
* **render:** render space-time diagrams incrementally ([b896437](https://github.com/mooofna/turing-machines/commit/b89643707c305db92fcfa99a85f36653ebba8a90))
* **wasm:** implement the bindings on top of the workspace crates ([7eee84f](https://github.com/mooofna/turing-machines/commit/7eee84fb59d87a781a67c2866706d149af956a1c))
* **web:** build the figures of chapters 4 to 6 ([b8fe634](https://github.com/mooofna/turing-machines/commit/b8fe6343cdde90e1e051ab9573aa5123ae3e132a))
* **web:** build the interactive figures of chapters 1 to 3 ([bba2887](https://github.com/mooofna/turing-machines/commit/bba288731f31a5601007130d21f616fb7eaaf567))
* **web:** build-time content pipeline, facts ledger and chapter layout ([f85c74d](https://github.com/mooofna/turing-machines/commit/f85c74da0b517e3d7b9c631cc7a9386ff7cad3a7))
* **web:** design system, site shell, landing, gallery and about pages ([d17e67a](https://github.com/mooofna/turing-machines/commit/d17e67a1463f7293c56a37591f47c9f48ea27389))
* **web:** run the playground on the real engine and describe every gallery machine ([f3aa38a](https://github.com/mooofna/turing-machines/commit/f3aa38a599bbab8b934086e7a6c18a29a1e6f4e8))
* **web:** set up-arrow values as maths and label formula table headers ([5461fff](https://github.com/mooofna/turing-machines/commit/5461fff17684b989657a3d50fd0c4900dc2d69a7))
* **web:** start the Antihydra figure on its own and continue the check on request ([f220cfe](https://github.com/mooofna/turing-machines/commit/f220cfe8548e67626e7b00286d4daf0949873ee4))


### Bug Fixes

* **bb:** make the busy beaver records independent of enumeration order ([bbb80df](https://github.com/mooofna/turing-machines/commit/bbb80dfd5eb0312cfcdd9497a324fc666fab797b))
* **cli:** shorten the check summary and correct help examples ([a04f113](https://github.com/mooofna/turing-machines/commit/a04f11340a4098adbd2f8c6f991c98031b00fc13))
* **core:** allow repeated stay steps in the run tape debug check ([86ae6db](https://github.com/mooofna/turing-machines/commit/86ae6dbf1d3290a164df0b3ef843400c485be34d))
* **core:** sharpen suggestions and spans of spec diagnostics ([e12a1f0](https://github.com/mooofna/turing-machines/commit/e12a1f0da5b469e0d7ed28a1913c996a32222856))
* **core:** write only final halting states as Z in the standard format ([e83cded](https://github.com/mooofna/turing-machines/commit/e83cded26a02fe43e943c245868eaa1fa1403d8a))
* **playground:** fit popovers and the diagram to their content ([334ddc9](https://github.com/mooofna/turing-machines/commit/334ddc95e37fcebdd2c3e454833d80ad90d1c325))
* **render:** keep labels from reading as one line ([0ae203b](https://github.com/mooofna/turing-machines/commit/0ae203b662aa5015b2413c1693c5866448a2d20c))
* **wasm:** check transition breakpoints before every step of a run ([0b31883](https://github.com/mooofna/turing-machines/commit/0b318833b54fb25772f4dafa3d1ec94b2388955e))
* **web:** count busy beaver states without the halting state in the gallery ([90d2809](https://github.com/mooofna/turing-machines/commit/90d280950ff968dd16766d447318f143f3075663))
* **web:** fit the figures of chapters 5 and 6 to phones and set their maths ([3546e13](https://github.com/mooofna/turing-machines/commit/3546e13286d9c8094d3b1cbb239869c4b954bdff))
* **web:** fit the machine count on phones and mark the hovered n in Fig. 1.3 ([d9136f0](https://github.com/mooofna/turing-machines/commit/d9136f0f437832fad11ae0cac2c9d699eca87e11))
* **web:** keep long exact diagrams inside their plate ([d3b2fb3](https://github.com/mooofna/turing-machines/commit/d3b2fb3ea4451222ec65c3054a19e6b175f3aead))
* **web:** keep the gallery table readable between phone and desktop widths ([f02dee0](https://github.com/mooofna/turing-machines/commit/f02dee0a2187cb77d58f77ae65db0ca4c97a9017))
* **web:** left-align displayed formulas and keep figures close to their section ([4a529bf](https://github.com/mooofna/turing-machines/commit/4a529bfd2ed67637f8cb112ff63b8b9c170cb0ef))
* **web:** read the diagram palette without transitions ([c3f200e](https://github.com/mooofna/turing-machines/commit/c3f200e31b8c16e3df5e43c9a53fdaaf16fbe16c))
* **web:** set chapter titles in the layout only, with the same separator as other pages ([e771790](https://github.com/mooofna/turing-machines/commit/e771790868de5c5549dd715390087d16bc2d57a4))


### Performance Improvements

* **core:** keep parsing and checking near linear on any input ([54c24bb](https://github.com/mooofna/turing-machines/commit/54c24bbbb1bafb732c37ab898ea258d569569168))
* **core:** speed up the dense single-tape hot loop ([a768e0e](https://github.com/mooofna/turing-machines/commit/a768e0e67b515058008dd39aad8aa7c9664909b4))
* **wasm:** optimise the module for size outside the hot crates ([d0fcdae](https://github.com/mooofna/turing-machines/commit/d0fcdae92834d5a7641c69830ce942137e1910bf))
* **wasm:** read the input DTOs without serde ([24bea52](https://github.com/mooofna/turing-machines/commit/24bea5269482f7b29679075980030c95bec80b62))
* **web:** size the chunks of the Antihydra check by time ([1966989](https://github.com/mooofna/turing-machines/commit/1966989185c93844d10435da01e6e429b5324cac))

## Changelog
