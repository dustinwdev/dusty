//! Maps element names to accesskit roles.

use accesskit::Role;

/// Maps a Dusty element name to the corresponding accesskit [`Role`].
///
/// # Examples
///
/// ```
/// use dusty_a11y::element_role;
/// use accesskit::Role;
///
/// assert_eq!(element_role("Button"), Role::Button);
/// assert_eq!(element_role("Text"), Role::Label);
/// assert_eq!(element_role("Unknown"), Role::GenericContainer);
/// ```
#[must_use]
pub fn element_role(name: &str) -> Role {
    match name {
        "Button" => Role::Button,
        "TextInput" | "Input" => Role::TextInput,
        "Checkbox" => Role::CheckBox,
        "Radio" => Role::RadioButton,
        "Toggle" | "Switch" => Role::Switch,
        "Slider" => Role::Slider,
        "ProgressBar" => Role::ProgressIndicator,
        "ScrollView" => Role::ScrollView,
        "Image" => Role::Image,
        "Divider" | "Separator" => Role::Splitter,
        "Row" | "Col" | "Column" | "Stack" => Role::Group,
        "Text" | "Label" => Role::Label,
        "Link" => Role::Link,
        "Dialog" | "Modal" => Role::Dialog,
        "Menu" => Role::Menu,
        "MenuItem" => Role::MenuItem,
        "Tab" => Role::Tab,
        "TabList" => Role::TabList,
        "TabPanel" => Role::TabPanel,
        "List" => Role::List,
        "ListItem" => Role::ListItem,
        "Heading" => Role::Heading,
        _ => Role::GenericContainer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_maps_to_button() {
        assert_eq!(element_role("Button"), Role::Button);
    }

    #[test]
    fn text_input_maps_to_text_input() {
        assert_eq!(element_role("TextInput"), Role::TextInput);
    }

    #[test]
    fn input_maps_to_text_input() {
        assert_eq!(element_role("Input"), Role::TextInput);
    }

    #[test]
    fn checkbox_maps_to_checkbox() {
        assert_eq!(element_role("Checkbox"), Role::CheckBox);
    }

    #[test]
    fn radio_maps_to_radio_button() {
        assert_eq!(element_role("Radio"), Role::RadioButton);
    }

    #[test]
    fn toggle_maps_to_switch() {
        assert_eq!(element_role("Toggle"), Role::Switch);
    }

    #[test]
    fn switch_maps_to_switch() {
        assert_eq!(element_role("Switch"), Role::Switch);
    }

    #[test]
    fn slider_maps_to_slider() {
        assert_eq!(element_role("Slider"), Role::Slider);
    }

    #[test]
    fn progress_bar_maps_to_progress_indicator() {
        assert_eq!(element_role("ProgressBar"), Role::ProgressIndicator);
    }

    #[test]
    fn scroll_view_maps_to_scroll_view() {
        assert_eq!(element_role("ScrollView"), Role::ScrollView);
    }

    #[test]
    fn image_maps_to_image() {
        assert_eq!(element_role("Image"), Role::Image);
    }

    #[test]
    fn divider_maps_to_splitter() {
        assert_eq!(element_role("Divider"), Role::Splitter);
    }

    #[test]
    fn separator_maps_to_splitter() {
        assert_eq!(element_role("Separator"), Role::Splitter);
    }

    #[test]
    fn row_maps_to_group() {
        assert_eq!(element_role("Row"), Role::Group);
    }

    #[test]
    fn col_maps_to_group() {
        assert_eq!(element_role("Col"), Role::Group);
    }

    #[test]
    fn column_maps_to_group() {
        assert_eq!(element_role("Column"), Role::Group);
    }

    #[test]
    fn stack_maps_to_group() {
        assert_eq!(element_role("Stack"), Role::Group);
    }

    #[test]
    fn text_maps_to_label() {
        assert_eq!(element_role("Text"), Role::Label);
    }

    #[test]
    fn label_maps_to_label() {
        assert_eq!(element_role("Label"), Role::Label);
    }

    #[test]
    fn link_maps_to_link() {
        assert_eq!(element_role("Link"), Role::Link);
    }

    #[test]
    fn dialog_maps_to_dialog() {
        assert_eq!(element_role("Dialog"), Role::Dialog);
    }

    #[test]
    fn modal_maps_to_dialog() {
        assert_eq!(element_role("Modal"), Role::Dialog);
    }

    #[test]
    fn menu_maps_to_menu() {
        assert_eq!(element_role("Menu"), Role::Menu);
    }

    #[test]
    fn menu_item_maps_to_menu_item() {
        assert_eq!(element_role("MenuItem"), Role::MenuItem);
    }

    #[test]
    fn tab_maps_to_tab() {
        assert_eq!(element_role("Tab"), Role::Tab);
    }

    #[test]
    fn tab_list_maps_to_tab_list() {
        assert_eq!(element_role("TabList"), Role::TabList);
    }

    #[test]
    fn tab_panel_maps_to_tab_panel() {
        assert_eq!(element_role("TabPanel"), Role::TabPanel);
    }

    #[test]
    fn list_maps_to_list() {
        assert_eq!(element_role("List"), Role::List);
    }

    #[test]
    fn list_item_maps_to_list_item() {
        assert_eq!(element_role("ListItem"), Role::ListItem);
    }

    #[test]
    fn heading_maps_to_heading() {
        assert_eq!(element_role("Heading"), Role::Heading);
    }

    #[test]
    fn unknown_maps_to_generic_container() {
        assert_eq!(element_role("CustomWidget"), Role::GenericContainer);
        assert_eq!(element_role("Foo"), Role::GenericContainer);
        assert_eq!(element_role(""), Role::GenericContainer);
    }
}
