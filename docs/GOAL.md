# Goal

The goal is a text AI that holds a free conversation in Polish and English: it answers in sentences of its own, not pasted excerpts, from what it holds, shows the source it relied on, and says it does not know when it holds nothing on the question.

The text AI is a goal, not a result. This file is the single statement of the
goal: the README, [start here](START-HERE.md) and the project website take their
wording from it, and a test keeps the README and start-here sentences equal to the
paragraph above. Each commitment below ends with one status line.

## 01 · Knowledge in memory, not weights

Knowledge is meant to go into memory, not into model weights: adding knowledge should need no fine-tuning.

Status: goal: knowledge updates without training.

## 02 · In its own words, from a source, or “I don’t know”

An answer is meant to be a plain sentence, not a pasted excerpt: it says it in its own words, shows the source it relied on, and when there is none, says plainly “I don’t know”.

Status: goal: answers in its own words, not yet measured; today the document tools return exact quotations.

## 03 · Free conversation in Polish and English

A conversation in both languages with no mode switching: the same knowledge bank, the same sources.

Status: goal: one conversation across both languages, not yet measured.

## 04 · Kept on disk

A copy kept on disk is meant to ensure that a restart or a process crash loses nothing confirmed as saved.

Status: process killed mid-write: 0 confirmed entries lost in the recorded crash series; not a power-cut test.
