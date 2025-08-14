use crate::models::{BlockStatus, Service, WorkStep};
use crate::system_state::SystemState;
use itertools::Itertools;
use ratatui::layout::Size;
use std::collections::HashMap;
use ratatui::prelude::Color;
use ui::component::{Component, Dir, Flow, FlowableArgs, List, MeasurableComponent, Space, Spinner, Text, WithMeasurement};
use ui::{FrameContext, RenderArgs, UIError, UIResult};
use crate::ui::theming::{ATTR_COLOR_WORK_ACTIVE, ATTR_COLOR_WORK_ERROR, ATTR_COLOR_WORK_IDLE, ATTR_COLOR_WORK_INACTIVE, ATTR_COLOR_WORK_PROCESSING, ATTR_COLOR_WORK_WAITING_TO_PROCESS};

pub struct ServiceList<'a> {
    pub state: &'a SystemState,
    pub show_selection: bool,
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
        let longest_name = services
            .iter()
            .map(|s| s.definition.id.len())
            .max()
            .unwrap_or(0);

        let idle_color = context.req_attr::<Color>(ATTR_COLOR_WORK_IDLE)?.clone();
        let inactive_color = context.req_attr::<Color>(ATTR_COLOR_WORK_INACTIVE)?.clone();
        let active_color = context.req_attr::<Color>(ATTR_COLOR_WORK_ACTIVE)?.clone();
        let processing_color = context.req_attr::<Color>(ATTR_COLOR_WORK_PROCESSING)?.clone();
        let waiting_color = context.req_attr::<Color>(ATTR_COLOR_WORK_WAITING_TO_PROCESS)?.clone();
        let error_color = context.req_attr::<Color>(ATTR_COLOR_WORK_ERROR)?.clone();

        context.render_component(RenderArgs::new(&List::new(
            &"view-profile-service-list",
            services,
            |service, _| {
                let block_statuses: HashMap<String, BlockUIStatus> = service
                    .definition
                    .blocks
                    .iter()
                    .map(|block| {
                        (
                            block.id.clone(),
                            match service.get_block_status(&block.id) {
                                BlockStatus::Initial => BlockUIStatus::Initial,
                                BlockStatus::Working { step } => match step {
                                    WorkStep::PrerequisiteCheck { last_failure, .. }
                                        if last_failure.is_some() =>
                                    {
                                        BlockUIStatus::FailedPrerequisites
                                    }
                                    _ => BlockUIStatus::Working,
                                },
                                BlockStatus::Ok => BlockUIStatus::Ok,
                                BlockStatus::Error => BlockUIStatus::Failed,
                                BlockStatus::Disabled => BlockUIStatus::Disabled,
                            },
                        )
                    })
                    .collect();
                let is_processing = block_statuses
                    .values()
                    .any(|status| matches!(status, BlockUIStatus::Working));

                let mut flow = Flow::new().dir(Dir::LeftRight);

                flow = flow.element(
                    Text::new(&service.definition.id).with_measurement(longest_name as u16, 1u16),
                    FlowableArgs { fill: true },
                );
                for slot in slots.iter() {
                    let block = service
                        .definition
                        .blocks
                        .iter()
                        .find(|b| b.status_line.slot == slot.order);
                    if let Some(block) = block {
                        flow = flow.element(
                            Text::new(&block.status_line.symbol)
                                .fg(match block_statuses.get(&block.id).unwrap() {
                                    BlockUIStatus::Initial => idle_color,
                                    BlockUIStatus::Disabled => inactive_color,
                                    BlockUIStatus::FailedPrerequisites => waiting_color,
                                    BlockUIStatus::Working => processing_color,
                                    BlockUIStatus::Ok => active_color,
                                    BlockUIStatus::Failed => error_color,
                                }),
                            FlowableArgs { fill: false },
                        );
                    } else {
                        flow = flow.element(
                            Text::new("-".repeat(slot.size)).fg(inactive_color),
                            FlowableArgs { fill: false },
                        );
                    }
                }

                flow = flow.element(
                    Text::new("O")
                        .fg(if service.output_enabled {
                            active_color
                        } else {
                            inactive_color
                        }),
                    FlowableArgs { fill: false },
                );
                // TODO account for automation state
                flow = flow.element(
                    Text::new("A").fg(active_color),
                    FlowableArgs { fill: false },
                );

                flow = flow.element(Spinner::new(is_processing), FlowableArgs { fill: false });

                Ok(flow)
            },
        ).highlight_visible(self.show_selection)))?;

        Ok(())
    }
}

impl<'a> MeasurableComponent for ServiceList<'_> {
    fn measure(&self, _context: &FrameContext) -> UIResult<Size> {
        let longest_name = self
            .services()?
            .iter()
            .map(|s| s.definition.id.len())
            .max()
            .unwrap_or(0);
        let slot_display_width = self.resolve_slots()?.iter().map(|s| s.size).sum::<usize>();

        Ok(Size {
            width: (longest_name + slot_display_width + 6) as u16,
            height: self.services()?.len() as u16,
        })
    }
}

struct SlotInfo {
    order: usize,
    size: usize,
}

pub enum BlockUIStatus {
    Initial,
    Disabled,
    Working,
    FailedPrerequisites,
    Failed,
    Ok,
}
