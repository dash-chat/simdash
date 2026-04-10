crate::routing_protocols! {
    Flooding / flooding => {
        model:  flooding::FloodingModel,
        state:  flooding::FloodingState,
        action: flooding::FloodingAction,
        fx:     flooding::FloodingFx,
        data:   flooding::FloodingData,
    },
}
