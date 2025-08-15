use crate::config::{ResolvedBlockActionBinding, ServiceActionBinding, ServiceActionTarget};
use crate::models::{BlockStatus, Service, WorkStep};
use crate::system_state::SystemState;
use crate::ui::inputs::ATTR_KEY_BLOCK_ACTIONS;
use crate::ui::theming::{
    ATTR_COLOR_WORK_ACTIVE, ATTR_COLOR_WORK_ERROR, ATTR_COLOR_WORK_IDLE, ATTR_COLOR_WORK_INACTIVE,
    ATTR_COLOR_WORK_PROCESSING, ATTR_COLOR_WORK_WAITING_TO_PROCESS,
};
use itertools::Itertools;
use ratatui::layout::Size;
use ratatui::prelude::Color;
use std::collections::HashMap;
use ui::component::{
    ATTR_KEY_NAV_DOWN, ATTR_KEY_NAV_TO_END, ATTR_KEY_NAV_TO_START, ATTR_KEY_NAV_UP,
    ATTR_KEY_SELECT, Component, Dir, Flow, FlowableArgs, List, MeasurableComponent, SimpleList,
    Space, Spinner, StatefulComponent, Text, WithMeasurement,
};
use ui::input::KeyMatcherQueryable;
use ui::{FrameContext, RenderArgs, UIError, UIResult};
use crate::ui::actions::{Action, ActionStore};

pub struct ServiceList<'a> {
    pub state: &'a SystemState,
    pub actions: &'a ActionStore,
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

#[derive(Default)]
pub struct ServiceListState {
    pub selection: usize,
}

impl<'a> StatefulComponent for ServiceList<'a> {
    type State = ServiceListState;
    type Output = ();

    fn state_id(&self) -> &str {
        "view-profile-service-list"
    }

    fn render(self, context: &mut FrameContext, state: &mut Self::State) -> UIResult<Self::Output> {
        let services = &self.services()?;
        let slots = self.resolve_slots()?;
        let longest_name = services
            .iter()
            .map(|s| s.definition.id.inner().len())
            .max()
            .unwrap_or(0);

        let idle_color = context.req_attr::<Color>(ATTR_COLOR_WORK_IDLE)?.clone();
        let inactive_color = context.req_attr::<Color>(ATTR_COLOR_WORK_INACTIVE)?.clone();
        let active_color = context.req_attr::<Color>(ATTR_COLOR_WORK_ACTIVE)?.clone();
        let processing_color = context
            .req_attr::<Color>(ATTR_COLOR_WORK_PROCESSING)?
            .clone();
        let waiting_color = context
            .req_attr::<Color>(ATTR_COLOR_WORK_WAITING_TO_PROCESS)?
            .clone();
        let error_color = context.req_attr::<Color>(ATTR_COLOR_WORK_ERROR)?.clone();

        if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_DOWN)?)
        {
            state.selection = (state.selection + 1).min(services.len());
        } else if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_UP)?)
        {
            state.selection = state.selection.saturating_sub(1);
        } else if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_TO_START)?)
        {
            state.selection = 0;
        } else if context
            .signals()
            .is_key_pressed(context.req_attr(ATTR_KEY_NAV_TO_END)?)
        {
            state.selection = services.len().saturating_sub(1);
        } else {
            // Sanity change here
            state.selection = state.selection.min(services.len());
            let block_actions =
                context.req_attr::<Vec<ResolvedBlockActionBinding>>(ATTR_KEY_BLOCK_ACTIONS)?;

            context
                .signals()
                .matching::<crossterm::event::KeyEvent>()
                .iter()
                .flat_map(|key_event| {
                    block_actions
                        .iter()
                        .filter(|action| action.keys.iter().any(|key| key.matches(key_event)))
                })
                .for_each(|action| {
                    let services = match action.target {
                        ServiceActionTarget::Selected => &services[state.selection..state.selection + 1],
                        ServiceActionTarget::All => services,
                    };
                    for service in services {
                        for block in &service.definition.blocks {
                            if action.blocks.iter().any(|block_id| {
                                block_id == "*" || block.id.inner() == block_id
                            }) {
                                self.actions.register(
                                    Action::SetBlockAction(
                                        service.definition.id.clone(),
                                        block.id.clone(),
                                        action.action.clone(),
                                    )
                                )
                            }
                        }
                    }
                });
        }

        context.render_component(RenderArgs::new(
            List::new(&"view-profile-service-list-list", services, |service, _| {
                let block_statuses: HashMap<String, BlockUIStatus> = service
                    .definition
                    .blocks
                    .iter()
                    .map(|block| {
                        (
                            block.id.inner().to_owned(),
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
                    Text::new(&service.definition.id.inner().to_owned()).with_measurement(longest_name as u16, 1u16),
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
                            Text::new(&block.status_line.symbol).fg(
                                match block_statuses.get(&block.id.inner().to_owned()).unwrap() {
                                    BlockUIStatus::Initial => idle_color,
                                    BlockUIStatus::Disabled => inactive_color,
                                    BlockUIStatus::FailedPrerequisites => waiting_color,
                                    BlockUIStatus::Working => processing_color,
                                    BlockUIStatus::Ok => active_color,
                                    BlockUIStatus::Failed => error_color,
                                },
                            ),
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
                    Text::new("O").fg(if service.output_enabled {
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
            })
            .selection(if self.show_selection {
                Some(state.selection)
            } else {
                None
            }),
        ))?;

        Ok(())
    }
}

impl<'a> MeasurableComponent for ServiceList<'_> {
    fn measure(&self, _context: &FrameContext) -> UIResult<Size> {
        let longest_name = self
            .services()?
            .iter()
            .map(|s| s.definition.id.inner().len())
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
