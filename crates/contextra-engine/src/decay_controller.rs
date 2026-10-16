// FILE-CONTEXT
// ZWECK: Re-Export des Adaptive Decay Controllers aus contextra-adapt (Ring 0/Adapt).
// INVARIANTEN: Re-export aller Decay-Controller Typen (AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs) ohne Verhalten/Definitionen zu duplizieren.

pub use contextra_adapt::decay_controller::*;
