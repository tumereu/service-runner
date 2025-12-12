use crate::config::BlockId;
use crate::models::{AutomationStatus, BlockStatus, WorkStep};
use crate::system_state::SystemState;
use crate::ui::inputs::{
    ATTR_KEY_BLOCK_ACTIONS, ATTR_KEY_TOGGLE_ALL_AUTOMATIONS, ATTR_KEY_TOGGLE_ALL_OUTPUT,
    ATTR_KEY_TOGGLE_SELECTED_AUTOMATIONS, ATTR_KEY_TOGGLE_SELECTED_OUTPUT,
};
use crate::ui::theming::{ATTR_COLOR_WORK_ACTIVE, ATTR_COLOR_WORK_ERROR, ATTR_COLOR_WORK_IDLE, ATTR_COLOR_WORK_INACTIVE, ATTR_COLOR_WORK_PARTIALLY_ACTIVE, ATTR_COLOR_WORK_PROCESSING, ATTR_COLOR_WORK_WAITING_TO_PROCESS};
use itertools::Itertools;
use ratatui::layout::Size;
use ratatui::prelude::Color;
use std::collections::HashMap;
use ui::component::{
    Dir, Flow, FlowableArgs, List, MeasurableComponent, Spinner,
    StatefulComponent, Text, WithMeasurement, ATTR_KEY_NAV_DOWN, ATTR_KEY_NAV_TO_END, ATTR_KEY_NAV_TO_START, ATTR_KEY_NAV_UP,
};
use ui::input::KeyMatcherQueryable;
use ui::{FrameContext, RenderArgs, UIError, UIResult};
use crate::config::{ResolvedBlockActionBinding, ServiceActionTarget};

pub struct ServiceList<'a> {
    pub system_state: &'a mut SystemState,
    pub show_selection: bool,
}
impl<'a> ServiceList<'a> {
    fn resolve_slots(&self) -> UIResult<Vec<SlotInfo>> {
        let services = &self.system_state
            .current_profile
            .as_ref()
            .ok_or(UIError::IllegalState {
                msg: "No profile selected".to_string(),
            })?
            .services;

        let mut size_by_slot: HashMap<usize, usize> = HashMap::new();
        for service in services {
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

    fn process_inputs(
        &mut self,
        context: &mut FrameContext,
        state: &mut ServiceListState,
    ) -> UIResult<()> {
        {
            let services = &self.system_state
                .current_profile
                .as_ref()
                .ok_or(UIError::IllegalState {
                    msg: "No profile selected".to_string(),
                })?
                .services;
            // List selection change/navigation
            if context
                .signals()
                .is_key_pressed(context.req_attr(ATTR_KEY_NAV_DOWN)?)
            {
                state.selection = state.selection.saturating_add(1);
            }
            if context
                .signals()
                .is_key_pressed(context.req_attr(ATTR_KEY_NAV_UP)?)
            {
                state.selection = state.selection.saturating_sub(1);
            }
            if context
                .signals()
                .is_key_pressed(context.req_attr(ATTR_KEY_NAV_TO_START)?)
            {
                state.selection = 0;
            }
            if context
                .signals()
                .is_key_pressed(context.req_attr(ATTR_KEY_NAV_TO_END)?)
            {
                state.selection = services.len().saturating_sub(1);
            }

            // Ensure the selection is within bounds
            state.selection = state.selection.min(services.len() - 1);
            let selection_id = services[state.selection].definition.id.clone();

            // Always available actions
            if context
                .signals()
                .is_key_pressed(context.req_attr(ATTR_KEY_TOGGLE_SELECTED_OUTPUT)?)
            {
                self.system_state.update_service(&selection_id, |service| {
                    service.output_enabled = !service.output_enabled;
                });
            }
            if context
                .signals()
                .is_key_pressed(context.req_attr(ATTR_KEY_TOGGLE_ALL_OUTPUT)?)
            {
                let any_enabled = self.system_state
                    .current_profile
                    .as_ref()
                    .unwrap()
                    .services
                    .iter()
                    .any(|service| service.output_enabled);
                self.system_state.update_all_services(|(_, service)| {
                    service.output_enabled = !any_enabled;
                });
            }
            if context
                .signals()
                .is_key_pressed(context.req_attr(ATTR_KEY_TOGGLE_SELECTED_AUTOMATIONS)?)
            {
                self.system_state.update_service(&selection_id, |service| {
                    service.automation_enabled = !service.automation_enabled;
                });
            }
            if context
                .signals()
                .is_key_pressed(context.req_attr(ATTR_KEY_TOGGLE_ALL_AUTOMATIONS)?)
            {
                let any_enabled = self.system_state
                    .current_profile
                    .as_ref()
                    .unwrap()
                    .services
                    .iter()
                    .any(|service| service.automation_enabled)
                    || self.system_state
                        .current_profile
                        .as_ref()
                        .map(|profile| profile.automation_enabled)
                        .unwrap_or(false);
                self.system_state.update_all_services(|(_, service)| {
                    service.automation_enabled = !any_enabled;
                });
                self.system_state.current_profile.iter_mut().for_each(|profile| {
                    profile.automation_enabled = !any_enabled;   
                })
            }
        }

        // User defined actions
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
                self.system_state.update_all_services(|(index, service)| {
                    let applies = match action.target {
                        ServiceActionTarget::Selected => index == state.selection,
                        ServiceActionTarget::All => true,
                    };

                    if applies {
                        let block_ids: Vec<BlockId> = service
                            .definition
                            .blocks
                            .iter()
                            .filter(|block| {
                                action
                                    .blocks
                                    .iter()
                                    .any(|block_id| block_id == block.id.inner())
                            })
                            .map(|block| block.id.clone())
                            .collect();

                        for block_id in block_ids {
                            service.update_block_action(&block_id, Some(action.action.clone()));
                        }
                    }
                });
            });

        Ok(())
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

    fn render(mut self, context: &mut FrameContext, state: &mut Self::State) -> UIResult<Self::Output> {
        self.process_inputs(context, state)?;

        let services = &self.system_state
            .current_profile
            .as_ref()
            .ok_or(UIError::IllegalState {
                msg: "No profile selected".to_string(),
            })?
            .services;
        let slots = self.resolve_slots()?;
        let longest_name = services
            .iter()
            .map(|s| s.definition.id.inner().len())
            .max()
            .unwrap_or(0);

        let idle_color = context.req_attr::<Color>(ATTR_COLOR_WORK_IDLE)?.clone();
        let inactive_color = context.req_attr::<Color>(ATTR_COLOR_WORK_INACTIVE)?.clone();
        let active_color = context.req_attr::<Color>(ATTR_COLOR_WORK_ACTIVE)?.clone();
        let partially_active_color = context.req_attr::<Color>(ATTR_COLOR_WORK_PARTIALLY_ACTIVE)?.clone();
        let processing_color = context
            .req_attr::<Color>(ATTR_COLOR_WORK_PROCESSING)?
            .clone();
        let waiting_color = context
            .req_attr::<Color>(ATTR_COLOR_WORK_WAITING_TO_PROCESS)?
            .clone();
        let error_color = context.req_attr::<Color>(ATTR_COLOR_WORK_ERROR)?.clone();

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
                                    WorkStep::ResourceGroupCheck { .. } => {
                                        BlockUIStatus::WaitingToProcess
                                    }
                                    WorkStep::PrerequisiteCheck { last_failure, .. }
                                        if last_failure.is_some() =>
                                    {
                                        BlockUIStatus::WaitingToProcess
                                    }
                                    _ => BlockUIStatus::Working,
                                },
                                BlockStatus::Ok { .. } => BlockUIStatus::Ok,
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
                    Text::new(&service.definition.id.inner().to_owned())
                        .with_measurement(longest_name as u16, 1u16),
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
                                    BlockUIStatus::WaitingToProcess => waiting_color,
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
                if service.automations.is_empty() {
                    flow = flow.element(
                        Text::new("-").fg(inactive_color),
                        FlowableArgs { fill: false },
                    );
                } else {
                    let has_automation_errors = service.automations
                        .iter()
                        .any(|automation| matches!(automation.status, AutomationStatus::Error));
                    let has_active_automations = service.automations
                        .iter()
                        .any(|automation| matches!(automation.status, AutomationStatus::Active));
                    let has_disabled_automations = service.automations
                        .iter()
                        .any(|automation| matches!(automation.status, AutomationStatus::Disabled));
                    let has_triggered_automations = service.automations
                        .iter()
                        .any(|automation| automation.last_triggered.is_some());

                    flow = flow.element(
                        Text::new("A")
                            .fg(
                                if !service.automation_enabled {
                                    inactive_color
                                } else if has_triggered_automations {
                                    processing_color
                                } else if has_automation_errors {
                                    error_color
                                } else if has_active_automations && has_disabled_automations {
                                    partially_active_color
                                } else if has_active_automations {
                                    active_color
                                } else {
                                    inactive_color
                                }
                            ),
                        FlowableArgs { fill: false },
                    );
                }

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

impl<'a> MeasurableComponent for ServiceList<'a> {
    fn measure(&self, _context: &FrameContext) -> UIResult<Size> {
        let services = &self.system_state
            .current_profile
            .as_ref()
            .ok_or(UIError::IllegalState {
                msg: "No profile selected".to_string(),
            })?
            .services;

        let longest_name = services
            .iter()
            .map(|s| s.definition.id.inner().len())
            .max()
            .unwrap_or(0);
        let slot_display_width = self.resolve_slots()?.iter().map(|s| s.size).sum::<usize>();

        Ok(Size {
            width: (longest_name + slot_display_width + 6) as u16,
            height: services.len() as u16,
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
    WaitingToProcess,
    Failed,
    Ok,
}
