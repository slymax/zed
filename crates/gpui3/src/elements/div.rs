use crate::{
    AnyElement, BorrowWindow, Bounds, Element, ElementFocus, ElementId, ElementInteraction,
    FocusDisabled, FocusEnabled, FocusHandle, FocusListeners, Focusable, GlobalElementId,
    GroupBounds, InteractiveElementState, IntoAnyElement, LayoutId, Overflow, ParentElement,
    Pixels, Point, SharedString, StatefulInteraction, StatefulInteractive, StatelessInteraction,
    StatelessInteractive, Style, StyleRefinement, Styled, ViewContext,
};
use parking_lot::Mutex;
use refineable::Refineable;
use smallvec::SmallVec;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct ScrollState(Arc<Mutex<Point<Pixels>>>);

impl ScrollState {
    pub fn x(&self) -> Pixels {
        self.0.lock().x
    }

    pub fn set_x(&self, value: Pixels) {
        self.0.lock().x = value;
    }

    pub fn y(&self) -> Pixels {
        self.0.lock().y
    }

    pub fn set_y(&self, value: Pixels) {
        self.0.lock().y = value;
    }
}

pub struct Div<
    V: 'static + Send + Sync,
    I: ElementInteraction<V> = StatelessInteraction<V>,
    F: ElementFocus<V> = FocusDisabled,
> {
    interaction: I,
    focus: F,
    children: SmallVec<[AnyElement<V>; 2]>,
    group: Option<SharedString>,
    base_style: StyleRefinement,
}

pub fn div<V>() -> Div<V, StatelessInteraction<V>, FocusDisabled>
where
    V: 'static + Send + Sync,
{
    Div {
        interaction: StatelessInteraction::default(),
        focus: FocusDisabled,
        children: SmallVec::new(),
        group: None,
        base_style: StyleRefinement::default(),
    }
}

impl<V, F> Div<V, StatelessInteraction<V>, F>
where
    F: ElementFocus<V>,
    V: 'static + Send + Sync,
{
    pub fn id(self, id: impl Into<ElementId>) -> Div<V, StatefulInteraction<V>, F> {
        Div {
            interaction: id.into().into(),
            focus: self.focus,
            children: self.children,
            group: self.group,
            base_style: self.base_style,
        }
    }
}

impl<V, I, F> Div<V, I, F>
where
    I: ElementInteraction<V>,
    F: ElementFocus<V>,
    V: 'static + Send + Sync,
{
    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn z_index(mut self, z_index: u32) -> Self {
        self.base_style.z_index = Some(z_index);
        self
    }

    pub fn overflow_hidden(mut self) -> Self {
        self.base_style.overflow.x = Some(Overflow::Hidden);
        self.base_style.overflow.y = Some(Overflow::Hidden);
        self
    }

    pub fn overflow_hidden_x(mut self) -> Self {
        self.base_style.overflow.x = Some(Overflow::Hidden);
        self
    }

    pub fn overflow_hidden_y(mut self) -> Self {
        self.base_style.overflow.y = Some(Overflow::Hidden);
        self
    }

    pub fn overflow_scroll(mut self, _scroll_state: ScrollState) -> Self {
        // todo!("impl scrolling")
        // self.scroll_state = Some(scroll_state);
        self.base_style.overflow.x = Some(Overflow::Scroll);
        self.base_style.overflow.y = Some(Overflow::Scroll);
        self
    }

    pub fn overflow_x_scroll(mut self, _scroll_state: ScrollState) -> Self {
        // todo!("impl scrolling")
        // self.scroll_state = Some(scroll_state);
        self.base_style.overflow.x = Some(Overflow::Scroll);
        self
    }

    pub fn overflow_y_scroll(mut self, _scroll_state: ScrollState) -> Self {
        // todo!("impl scrolling")
        // self.scroll_state = Some(scroll_state);
        self.base_style.overflow.y = Some(Overflow::Scroll);
        self
    }

    fn with_element_id<R>(
        &mut self,
        cx: &mut ViewContext<V>,
        f: impl FnOnce(&mut Self, Option<GlobalElementId>, &mut ViewContext<V>) -> R,
    ) -> R {
        if let Some(id) = self.id() {
            cx.with_element_id(id, |global_id, cx| f(self, Some(global_id), cx))
        } else {
            f(self, None, cx)
        }
    }

    pub fn compute_style(
        &self,
        bounds: Bounds<Pixels>,
        state: &InteractiveElementState,
        cx: &mut ViewContext<V>,
    ) -> Style {
        let mut computed_style = Style::default();
        computed_style.refine(&self.base_style);
        self.focus.refine_style(&mut computed_style, cx);
        self.interaction
            .refine_style(&mut computed_style, bounds, state, cx);
        computed_style
    }
}

impl<V, I> Div<V, I, FocusDisabled>
where
    I: ElementInteraction<V>,
    V: 'static + Send + Sync,
{
    pub fn focusable(self, handle: &FocusHandle) -> Div<V, I, FocusEnabled<V>> {
        Div {
            interaction: self.interaction,
            focus: handle.clone().into(),
            children: self.children,
            group: self.group,
            base_style: self.base_style,
        }
    }
}

impl<V, I> Focusable for Div<V, I, FocusEnabled<V>>
where
    I: ElementInteraction<V>,
    V: 'static + Send + Sync,
{
    fn focus_listeners(&mut self) -> &mut FocusListeners<V> {
        &mut self.focus.focus_listeners
    }

    fn handle(&self) -> &FocusHandle {
        &self.focus.focus_handle
    }

    fn set_focus_style(&mut self, style: StyleRefinement) {
        self.focus.focus_style = style;
    }

    fn set_focus_in_style(&mut self, style: StyleRefinement) {
        self.focus.focus_in_style = style;
    }

    fn set_in_focus_style(&mut self, style: StyleRefinement) {
        self.focus.in_focus_style = style;
    }
}

impl<V, I, F> Element for Div<V, I, F>
where
    I: ElementInteraction<V>,
    F: ElementFocus<V>,
    V: 'static + Send + Sync,
{
    type ViewState = V;
    type ElementState = InteractiveElementState;

    fn id(&self) -> Option<ElementId> {
        self.interaction
            .as_stateful()
            .map(|identified| identified.id.clone())
    }

    fn initialize(
        &mut self,
        view_state: &mut Self::ViewState,
        element_state: Option<Self::ElementState>,
        cx: &mut ViewContext<Self::ViewState>,
    ) -> Self::ElementState {
        self.interaction.initialize(cx, |cx| {
            self.focus.initialize(cx, |cx| {
                for child in &mut self.children {
                    child.initialize(view_state, cx);
                }
            });
        });
        element_state.unwrap_or_default()
    }

    fn layout(
        &mut self,
        view_state: &mut Self::ViewState,
        element_state: &mut Self::ElementState,
        cx: &mut ViewContext<Self::ViewState>,
    ) -> LayoutId {
        let style = self.compute_style(Bounds::default(), element_state, cx);
        style.apply_text_style(cx, |cx| {
            self.with_element_id(cx, |this, _global_id, cx| {
                let layout_ids = this
                    .children
                    .iter_mut()
                    .map(|child| child.layout(view_state, cx))
                    .collect::<Vec<_>>();
                cx.request_layout(&style, layout_ids)
            })
        })
    }

    fn paint(
        &mut self,
        bounds: Bounds<Pixels>,
        view_state: &mut Self::ViewState,
        element_state: &mut Self::ElementState,
        cx: &mut ViewContext<Self::ViewState>,
    ) {
        self.with_element_id(cx, |this, _global_id, cx| {
            if let Some(group) = this.group.clone() {
                GroupBounds::push(group, bounds, cx);
            }

            let style = this.compute_style(bounds, element_state, cx);
            let z_index = style.z_index.unwrap_or(0);

            // Paint background and event handlers.
            cx.stack(z_index, |cx| {
                cx.stack(0, |cx| {
                    style.paint(bounds, cx);

                    this.focus.paint(bounds, cx);
                    this.interaction.paint(bounds, element_state, cx);
                });

                cx.stack(1, |cx| {
                    style.apply_text_style(cx, |cx| {
                        style.apply_overflow(bounds, cx, |cx| {
                            for child in &mut this.children {
                                child.paint(view_state, None, cx);
                            }
                        })
                    })
                });
            });

            if let Some(group) = this.group.as_ref() {
                GroupBounds::pop(group, cx);
            }
        })
    }
}

impl<V, I, F> IntoAnyElement<V> for Div<V, I, F>
where
    I: ElementInteraction<V>,
    F: ElementFocus<V>,
    V: 'static + Send + Sync,
{
    fn into_any(self) -> AnyElement<V> {
        AnyElement::new(self)
    }
}

impl<V, I, F> ParentElement for Div<V, I, F>
where
    I: ElementInteraction<V>,
    F: ElementFocus<V>,
    V: 'static + Send + Sync,
{
    fn children_mut(&mut self) -> &mut SmallVec<[AnyElement<Self::ViewState>; 2]> {
        &mut self.children
    }
}

impl<V, I, F> Styled for Div<V, I, F>
where
    I: ElementInteraction<V>,
    F: ElementFocus<V>,
    V: 'static + Send + Sync,
{
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.base_style
    }
}

impl<V, I, F> StatelessInteractive for Div<V, I, F>
where
    I: ElementInteraction<V>,
    F: ElementFocus<V>,
    V: 'static + Send + Sync,
{
    fn stateless_interactivity(&mut self) -> &mut StatelessInteraction<V> {
        self.interaction.as_stateless_mut()
    }
}

impl<V, F> StatefulInteractive for Div<V, StatefulInteraction<V>, F>
where
    F: ElementFocus<V>,
    V: 'static + Send + Sync,
{
    fn stateful_interactivity(&mut self) -> &mut StatefulInteraction<Self::ViewState> {
        &mut self.interaction
    }
}
