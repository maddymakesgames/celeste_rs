use crate::maps::{
    ErasedMapElement,
    LookupTable,
    MapAttribute,
    RawMapElement,
    ResolvableString,
    var_types::EncodedVar,
};

pub struct MapEncoder<'a> {
    pub lookup: &'a mut LookupTable,
    pub(crate) element_name: ResolvableString,
    pub(crate) children: Vec<RawMapElement>,
    pub(crate) attrs: Vec<MapAttribute>,
}

impl MapEncoder<'_> {
    /// Sets the name of the raw element
    ///
    /// This is already done when creating the [MapEncoder] so this
    /// is only needed when [MapElement::NAME](crate::maps::MapElement::NAME) is incorrect
    pub fn name(&mut self, name: impl AsRef<str>) {
        self.element_name = self.lookup.index_string(name);
    }

    /// Pushes a new [MapAttribute] onto the raw element
    pub fn attribute(&mut self, name: impl AsRef<str>, value: impl Into<EncodedVar>) {
        self.attrs
            .push(MapAttribute::new(self.lookup.index_string(name), value))
    }

    /// Pushes a new [MapAttribute] onto the raw element if the option value is `Some`
    pub fn optional_attribute<'c, T: Into<EncodedVar> + Clone + 'c>(
        &mut self,
        name: impl AsRef<str>,
        value: &Option<T>,
    ) {
        if let Some(t) = value {
            self.attribute(name, t.clone())
        }
    }

    /// Pushes a new child [MapElement](crate::maps::MapElement) onto the raw element
    pub fn child<T: ErasedMapElement>(&mut self, child: &T) {
        let child_name = self.lookup.index_string(child.name());
        let mut fork = self.fork(child_name);

        child.to_raw(&mut fork);

        let child = fork.resolve();

        self.children.push(child);
    }

    /// Pushes all the [MapElement](crate::maps::MapElement)s in the list as children on the raw elements
    pub fn children<T: ErasedMapElement>(&mut self, children: impl AsRef<[T]>) {
        let children = children.as_ref();

        for child in children {
            self.child(child)
        }
    }

    /// Forks the current encoder to create a new [RawMapElement]
    #[doc(hidden)]
    pub fn fork(&mut self, name: ResolvableString) -> MapEncoder {
        MapEncoder {
            lookup: self.lookup,
            element_name: name,
            children: Vec::new(),
            attrs: Vec::new(),
        }
    }

    /// Consumes the [MapEncoder] converting it into a [RawMapElement]
    pub fn resolve(self) -> RawMapElement {
        RawMapElement {
            name: self.element_name,
            attributes: self.attrs,
            children: self.children,
        }
    }

    /// Loads data from an existing [RawMapElement]
    #[doc(hidden)]
    pub fn from_raw(&mut self, raw: &RawMapElement) {
        self.element_name = raw.name.clone();
        self.attrs.clone_from(&raw.attributes);
        self.children.clone_from(&raw.children);
    }
}
