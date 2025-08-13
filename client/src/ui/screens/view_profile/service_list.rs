use crate::models::Service;
use crate::system_state::SystemState;
use itertools::Itertools;
use ratatui::layout::Size;
use std::collections::HashMap;
use ui::component::{Component, Dir, Flow, FlowableArgs, List, MeasurableComponent, Space, Text, WithMeasurement};
use ui::{FrameContext, RenderArgs, UIError, UIResult};

pub struct ServiceList<'a> {
    pub state: &'a SystemState,
}
impl ServiceList<'_> {
    pub fn services(&self) -> UIResult<&Vec<Service>> {
        Ok(&self
            .state
            .current_profile
            .as_ref()
            .ok_or(UIError::IllegalState {
                msg: "No profile selected".to_string(),
            })?
            .services)
    }

    pub fn resolve_slots(&self) -> UIResult<Vec<SlotInfo>> {
        let mut size_by_slot: HashMap<usize, usize> = HashMap::new();
        for service in self.services()? {
            for block in &service.definition.blocks {
                let existing = *size_by_slot.get(&block.status_line.slot).unwrap_or(&0);
                size_by_slot.insert(
                    block.status_line.slot,
                    block.status_line.symbol.len().max(existing),
                );
            }
        }

        Ok(size_by_slot
            .iter()
            .map(|(slot, size)| SlotInfo {
                order: *slot,
                size: *size,
            })
            .sorted_by_key(|s| s.order)
            .collect())
    }
}

impl<'a> Component for ServiceList<'a> {
    type Output = ();

    fn render(&self, context: &mut FrameContext) -> UIResult<Self::Output> {
        let services = &self.services()?;
        let slots = self.resolve_slots()?;
        let longest_name = services.iter().map(|s| s.definition.id.len()).max().unwrap_or(0);

        context.render_component(RenderArgs::new(&List::new(
            &"view-profile-service-list",
            services,
            |service, _| {
                let mut flow = Flow::new().dir(Dir::LeftRight);

                flow = flow.element(
                    Text::new(&service.definition.id).with_measurement(longest_name as u16, 1u16),
                    FlowableArgs { fill: false }
                );
                flow = flow.element(Space::new(1u16, 1u16), FlowableArgs { fill: false });
                for slot in slots.iter() {
                    let block = service.definition.blocks.iter().find(|b| b.status_line.slot == slot.order);
                    if let Some(block) = block {
                        // TODO colors?
                        flow = flow.element(Text::new(&block.status_line.symbol), FlowableArgs { fill: false });
                    } else {
                        flow = flow.element(Space::new(slot.size as u16, 1u16), FlowableArgs { fill: false });
                    }
                }

                flow
            }

        )))?;

        Ok(())
    }
}

impl<'a> MeasurableComponent for ServiceList<'_> {
    fn measure(&self, _context: &FrameContext) -> UIResult<Size> {
        let longest_name = self.services()?.iter().map(|s| s.definition.id.len()).max().unwrap_or(0);
        let slot_display_width = self.resolve_slots()?.iter().map(|s| s.size).sum::<usize>();

        Ok(Size {
            width: (longest_name + slot_display_width + 3) as u16,
            height: self.services()?.len() as u16,
        })
    }
}

struct SlotInfo {
    order: usize,
    size: usize,
}
