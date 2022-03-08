use crate::model::{AnnotatedAxiom, Axiom, MutableOntology, ForIRI};

use super::indexed::{rc_unwrap_or_clone, OntologyIndex, ThreeIndexedOntology, TwoIndexedOntology};
use std::collections::HashMap;
use std::convert::AsRef;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct LogicallyEqualIndex<A: ForIRI>(HashMap<Axiom<A>, Rc<AnnotatedAxiom<A>>>);

impl<A: ForIRI> OntologyIndex<A> for LogicallyEqualIndex<A> {
    fn index_insert(&mut self, ax: Rc<AnnotatedAxiom<A>>) -> bool {
        self.0.insert(ax.axiom.clone(), ax).is_some()
    }

    fn index_take(&mut self, ax: &AnnotatedAxiom<A>) -> Option<AnnotatedAxiom<A>> {
        self.0.remove(&ax.axiom).map(rc_unwrap_or_clone)
    }

    fn index_remove(&mut self, ax: &AnnotatedAxiom<A>) -> bool {
        self.0.remove(&ax.axiom).is_some()
    }
}

impl<A: ForIRI> LogicallyEqualIndex<A> {
    pub fn logical_contains(&self, ax: &AnnotatedAxiom<A>) -> bool {
        self.0.contains_key(&ax.axiom)
    }

    pub fn logical_get(&self, ax: &AnnotatedAxiom<A>) -> Option<&AnnotatedAxiom<A>> {
        self.0.get(&ax.axiom).map(|rcax| &**rcax)
    }

    pub fn logical_get_rc(&self, ax: &AnnotatedAxiom<A>) -> Option<Rc<AnnotatedAxiom<A>>> {
        self.0.get(&ax.axiom).cloned()
    }
}

impl<A: ForIRI, I: OntologyIndex<A>> AsRef<LogicallyEqualIndex<A>>
    for TwoIndexedOntology<A, I, LogicallyEqualIndex<A>> {
    fn as_ref(&self) -> &LogicallyEqualIndex<A> {
        self.j()
    }
}

impl<A: ForIRI, I, J> AsRef<LogicallyEqualIndex<A>>
    for ThreeIndexedOntology<A, I, J, LogicallyEqualIndex<A>>
where
    I: OntologyIndex<A>,
    J: OntologyIndex<A>,
{
    fn as_ref(&self) -> &LogicallyEqualIndex<A> {
        self.k()
    }
}

pub fn update_or_insert_logically_equal_axiom<'a, A: ForIRI, O>(o: &mut O, axiom: AnnotatedAxiom<A>)
where
    O: MutableOntology<A> + AsRef<LogicallyEqualIndex<A>>,
{
    if let Some(axiom) = update_logically_equal_axiom(o, axiom) {
        o.insert(axiom);
    }
}

pub fn update_logically_equal_axiom<'a, A: ForIRI, O>(
    o: &mut O,
    mut axiom: AnnotatedAxiom<A>,
) -> Option<AnnotatedAxiom<A>>
where
    O: MutableOntology<A> + AsRef<LogicallyEqualIndex<A>>,
{
    let lei: &LogicallyEqualIndex<_> = o.as_ref();
    let src = lei.logical_get_rc(&axiom);
    // Does the logically equal axiom exist
    if let Some(rc) = src {
        // Remove the rc from everywhere
        o.remove(&*rc);
        //dbg!(&rc);
        //dbg!(Rc::strong_count(&rc));

        // Un-rc
        let mut logical_axiom = Rc::try_unwrap(rc).unwrap();
        // Extend it
        logical_axiom.ann.append(&mut axiom.ann);
        // Insert it
        o.insert(logical_axiom);
        None
    } else {
        // Otherwise put the one we have in
        Some(axiom)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::model::*;
    use crate::ontology::indexed::*;
    use crate::ontology::set::*;

    #[test]
    fn cons() {
        let _lei = LogicallyEqualIndex::default();
        assert!(true);
    }

    #[test]
    fn equal_retrieve() {
        // Setup
        let build = Build::new();
        let mut o = LogicallyEqualIndex::default();
        let decl1: AnnotatedAxiom = DeclareClass(build.class("http://www.example.com#a")).into();
        let decl2: AnnotatedAxiom = DeclareClass(build.class("http://www.example.com#b")).into();
        let decl3: AnnotatedAxiom = DeclareClass(build.class("http://www.example.com#c")).into();

        o.index_insert(Rc::new(decl1.clone()));
        o.index_insert(Rc::new(decl2.clone()));
        o.index_insert(Rc::new(decl3.clone()));

        assert!(o.logical_contains(&decl1));
        assert!(o.logical_contains(&decl2));
        assert!(o.logical_contains(&decl3));
    }

    #[test]
    fn annotation_not_equal_retrieve() {
        // Setup
        let b = Build::new();
        let mut o: TwoIndexedOntology<SetIndex, LogicallyEqualIndex> = Default::default();

        let ann = Annotation {
            ap: b.annotation_property("http://www.example.com/ap"),
            av: b.iri("http://www.example.com/av").into(),
        };

        let decl1: AnnotatedAxiom = DeclareClass(b.class("http://www.example.com#a")).into();
        let decl2: AnnotatedAxiom = DeclareClass(b.class("http://www.example.com#b")).into();
        let decl3: AnnotatedAxiom = DeclareClass(b.class("http://www.example.com#c")).into();

        let mut decl1_a = decl1.clone();
        decl1_a.ann.insert(ann.clone());

        let mut decl2_a = decl2.clone();
        decl2_a.ann.insert(ann);

        o.insert(decl1_a.clone());
        o.insert(decl2_a.clone());
        o.insert(decl3.clone());

        assert!(!o.i().contains(&decl1));
        assert!(!o.i().contains(&decl2));
        assert!(o.i().contains(&decl3));

        assert!(o.j().logical_contains(&decl1));
        assert!(o.j().logical_contains(&decl2));
        assert!(o.j().logical_contains(&decl3));
    }

    #[test]
    fn test_update_equal_axiom() {
        let b = Build::new();
        {
            let mut o: TwoIndexedOntology<SetIndex, LogicallyEqualIndex> = Default::default();
            let ne: NamedEntity = b.class("http://www.example.com").into();
            let ax: Axiom = ne.into();
            let mut dec: AnnotatedAxiom = ax.into();

            dec.ann.insert(Annotation {
                ap: b.annotation_property("http://www.example.com/p1"),
                av: b.iri("http://www.example.com/a1").into(),
            });

            let ne: NamedEntity = b.class("http://www.example.com").into();
            let ax: Axiom = ne.into();
            let mut dec2: AnnotatedAxiom = ax.into();

            dec2.ann.insert(Annotation {
                ap: b.annotation_property("http://www.example.com/p1"),
                av: b.iri("http://www.example.com/a2").into(),
            });

            o.insert(dec);
            o.insert(dec2);
            assert_eq!(o.i().into_iter().count(), 2);
        }

        {
            let mut o: TwoIndexedOntology<SetIndex, LogicallyEqualIndex> = Default::default();
            let ne: NamedEntity = b.class("http://www.example.com").into();
            let ax: Axiom = ne.into();
            let mut dec: AnnotatedAxiom = ax.into();
            dec.ann.insert(Annotation {
                ap: b.annotation_property("http://www.example.com/p1"),
                av: b.iri("http://www.example.com/a1").into(),
            });

            let ne: NamedEntity = b.class("http://www.example.com").into();
            let ax: Axiom = ne.into();
            let mut dec2: AnnotatedAxiom = ax.into();
            dec2.ann.insert(Annotation {
                ap: b.annotation_property("http://www.example.com/p1"),
                av: b.iri("http://www.example.com/a2").into(),
            });

            o.insert(dec);
            update_logically_equal_axiom(&mut o, dec2);
            assert_eq!(o.i().into_iter().count(), 1);

            let aa = o.i().into_iter().next().unwrap();

            assert_eq!(aa.ann.iter().count(), 2);
        }
    }
}
