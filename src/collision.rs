#![allow(unused)]

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::BTreeMap;

fn float_max(a : f64, b : f64) -> f64
{
    if a == b && a.signum() == b.signum() // discriminate positive and negative 0
    {
        a
    }
    else if a > b || a.signum() > b.signum()
    {
        a
    }
    else if b > a || b.signum() > a.signum()
    {
        b
    }
    // NAN territory below
    else if a.is_nan()
    {
        b
    }
    else
    {
        a
    }
}
fn float_min(a : f64, b : f64) -> f64
{
    if a == b && a.signum() == b.signum() // discriminate positive and negative 0
    {
        a
    }
    else if a < b || a.signum() < b.signum()
    {
        a
    }
    else if b < a || b.signum() < a.signum()
    {
        b
    }
    // NAN territory below
    else if a.is_nan()
    {
        b
    }
    else
    {
        a
    }
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x : f64,
    y : f64
}
#[allow(dead_code)]
impl Point {
    fn zero() -> Point
    {
        Point{x : 0.0, y : 0.0}
    }
    fn from(x : f64, y : f64) -> Point
    {
        Point{x, y}
    }
    fn dot(&self, other : &Point) -> Point
    {
        Point{x : self.x*other.x, y : self.y*other.y}
    }
    fn normalize(&self) -> Point
    {
        let magnitude = self.magnitude();
        if magnitude == 0.0
        {
            return self.clone();
        }
        Point{x : self.x/magnitude, y : self.y/magnitude}
    }
    fn magnitude_square(&self) -> f64
    {
        self.x*self.x + self.y*self.y
    }
    fn magnitude(&self) -> f64
    {
        self.magnitude_square().sqrt()
    }
    fn dot_mut<'a>(&'a mut self, other : &Point) -> &'a mut Point
    {
        self.x *= other.x;
        self.y *= other.y;
        self
    }
    fn normalize_mut<'a>(&'a mut self) -> &'a mut Point
    {
        let magnitude = self.magnitude();
        if magnitude == 0.0
        {
            return self;
        }
        self.x /= magnitude;
        self.y /= magnitude;
        self
    }
    fn add(self, other : &Point) -> Point
    {
        Point{x : self.x + other.x, y : self.y + other.y}
    }
    fn sub(self, other : &Point) -> Point
    {
        Point{x : self.x - other.x, y : self.y - other.y}
    }
    fn add_mut<'a>(&'a mut self, other : &Point) -> &'a mut Point
    {
        self.x += other.x;
        self.y += other.y;
        self
    }
    fn sub_mut<'a>(&'a mut self, other : &Point) -> &'a mut Point
    {
        self.x -= other.x;
        self.y -= other.y;
        self
    }
    fn minima(&self, other : &Point) -> Point
    {
        Point{x : float_min(self.x, other.x), y : float_min(self.y, other.y)}
    }
    fn maxima(&self, other : &Point) -> Point
    {
        Point{x : float_max(self.x, other.x), y : float_max(self.y, other.y)}
    }
    fn minima_mut<'a>(&'a mut self, other : &Point) -> &'a mut Point
    {
        self.x = float_min(self.x, other.x);
        self.y = float_min(self.y, other.y);
        self
    }
    fn maxima_mut<'a>(&'a mut self, other : &Point) -> &'a mut Point
    {
        self.x = float_max(self.x, other.x);
        self.y = float_max(self.y, other.y);
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct AABB {
    minima : Point,
    maxima : Point
}

impl AABB {
    fn bvh_heuristic(&self) -> f64
    {
        let w = self.maxima.x - self.minima.x;
        let h = self.maxima.y - self.minima.y;
        w*2.0 + h*2.0
        //w*h
        //w*w + h*h
        //float_min(w, h)
        //float_max(w, h)
    }
    fn translate(&self, add : &Point) -> AABB
    {
        AABB{minima : self.minima.add(add), maxima : self.maxima.add(add)}
    }
    fn union(&self, other : &AABB) -> AABB
    {
        AABB{minima : self.minima.minima(&other.minima), maxima : self.maxima.maxima(&other.maxima)}
    }
    fn union_mut<'a>(&'a mut self, other : &AABB) -> &'a mut AABB
    {
        self.minima.minima_mut(&other.minima);
        self.maxima.maxima_mut(&other.maxima);
        self
    }
    fn fatten(&self) -> AABB
    {
        AABB {
            minima : self.minima.sub(&Point{x:1.0, y:1.0}),
            maxima : self.maxima.add(&Point{x:1.0, y:1.0})
        }
    }
    fn from_points(points : &[Point]) -> AABB
    {
        let mut ret = AABB {
            minima : points[0],
            maxima : points[0]
        };
        for point in &points[1..]
        {
            ret.minima.minima_mut(point);
            ret.maxima.maxima_mut(point);
        }
        ret
    }
    fn from_radius(radius : f64) -> AABB
    {
        AABB {
            minima : Point{x : -radius, y : -radius},
            maxima : Point{x :  radius, y :  radius},
        }
    }
    fn contains(&self, other : &AABB) -> bool
    {
        self.minima.x <= other.minima.x &&
        self.minima.y <= other.minima.y &&
        self.maxima.x >= other.maxima.x &&
        self.maxima.y >= other.maxima.y
    }
    fn contained_by(&self, other : &AABB) -> bool
    {
        other.contains(self)
    }
    fn dump_rects(&self, depth : usize) -> String
    {
        format!("[{}, {}, {}, {}, {}],", depth, self.minima.x, self.minima.y, self.maxima.x-self.minima.x, self.maxima.y-self.minima.y)
    }
}

#[derive(Clone, Debug)]
struct Polygon {
    points : Vec<Point>
}

#[derive(Clone, Debug)]
struct Circle {
    origin_offset : Point, // point added to object origin to get origin of circle
    radius : f64
}

#[derive(Clone, Debug)]
enum Shape {
    Poly(Polygon),
    Circle(Circle)
}

impl Shape {
    fn calculate_raw_aabb(&self) -> AABB
    {
        match self
        {
            Shape::Poly(polygon) => AABB::from_points(&polygon.points),
            Shape::Circle(circle) => AABB::from_radius(circle.radius).translate(&circle.origin_offset)
        }
    }
}

#[derive(Debug)]
struct PositionedShape {
    aabb_raw : AABB,
    aabb_positioned : AABB,
    shape : Shape,
    origin : Point,
    parent : Weak<RefCell<TreeNode>>,
    id : u64,
    is_static : bool,
}

type NodeRef = Rc<RefCell<TreeNode>>;
type ShapeRef = Rc<RefCell<PositionedShape>>;

#[derive(Debug)]
enum TreeChild {
    Nodes([NodeRef; 2]),
    Shape(ShapeRef)
}

impl TreeChild {
    fn is_shape(&self) -> bool
    {
        if let TreeChild::Shape(_) = self { true } else { false }
    }
}

#[derive(Debug)]
struct TreeNode {
    bounds : AABB,
    child : TreeChild,
    parent : Weak<RefCell<TreeNode>>,
    leafs : usize,
    depth : usize,
}

fn recalculate_aabb_shape(target : &ShapeRef)
{
    let mut target = target.borrow_mut();
    target.aabb_positioned = target.aabb_positioned.translate(&target.origin);
    if let Some(target) = &target.parent.upgrade()
    {
        recalculate_aabb_node(&target);
    }
}
fn recalculate_aabb_node(target : &NodeRef)
{
    let mut inner_target = target.borrow_mut();
    inner_target.bounds = match &inner_target.child
    {
        TreeChild::Nodes(nodes) => nodes[0].borrow().bounds.union(&nodes[1].borrow().bounds).fatten(),
        TreeChild::Shape(shape) => shape.borrow().aabb_positioned.fatten()
    };
    drop(inner_target);
    if let Some(target) = &target.borrow().parent.upgrade()
    {
        recalculate_aabb_node(&target);
    }
}
fn calculate_shared_bvh_heuristic(left : &NodeRef, right : &NodeRef) -> f64
{
    left.borrow().bounds.union(&right.borrow().bounds).bvh_heuristic()
}

fn new_node_from_nodes(left : NodeRef, right : NodeRef) -> NodeRef
{
    let left_borrowed = left.borrow();
    let right_borrowed = right.borrow();
    let bounds = left_borrowed.bounds.union(&right_borrowed.bounds).fatten();
    let leafs = left_borrowed.leafs + right_borrowed.leafs;
    let depth = 1 + std::cmp::max(left_borrowed.depth, right_borrowed.depth);
    
    drop(left_borrowed);
    drop(right_borrowed);
    
    let ret = Rc::new(RefCell::new(TreeNode {
        bounds,
        child : TreeChild::Nodes([left, right]),
        parent : Weak::new(),
        leafs,
        depth,
    }));
    
    if let TreeChild::Nodes(nodes) = &ret.borrow().child
    {
        nodes[0].borrow_mut().parent = Rc::downgrade(&ret);
        nodes[1].borrow_mut().parent = Rc::downgrade(&ret);
    }
    
    ret
}
fn new_node_from_shape(shape : PositionedShape) -> NodeRef
{
    let bounds = shape.aabb_positioned.fatten();
    
    let ret = Rc::new(RefCell::new(TreeNode {
        bounds,
        child : TreeChild::Shape(Rc::new(RefCell::new(shape))),
        parent : Weak::new(),
        leafs : 1,
        depth : 1,
    }));
    
    if let TreeChild::Shape(shape) = &ret.borrow().child
    {
        shape.borrow_mut().parent = Rc::downgrade(&ret);
    }
    
    ret
}

const ALLOWED_DEPTH_DISBALANCE : usize = 1;

fn rebalance_internal(parent : &mut NodeRef) -> bool
{
    let new_parent;
    if let TreeChild::Nodes(nodes) = &parent.borrow().child
    {
        let short_node;
        let tall_node;
        if nodes[0].borrow().depth+ALLOWED_DEPTH_DISBALANCE < nodes[1].borrow().depth
        {
            short_node = Rc::clone(&nodes[0]);
            tall_node  = Rc::clone(&nodes[1]);
        }
        else if nodes[0].borrow().depth > nodes[1].borrow().depth+ALLOWED_DEPTH_DISBALANCE
        {
            short_node = Rc::clone(&nodes[1]);
            tall_node  = Rc::clone(&nodes[0]);
        }
        else
        {
            return false;
        }
        if let TreeChild::Nodes(nodes) = &tall_node.borrow().child
        {
            // FIXME: is this the right way to do this?
            let left_shorter = nodes[0].borrow().depth+(ALLOWED_DEPTH_DISBALANCE-1) < nodes[1].borrow().depth;
            let right_shorter = nodes[0].borrow().depth > nodes[1].borrow().depth+(ALLOWED_DEPTH_DISBALANCE-1);
            if left_shorter
            {
                let new_child = new_node_from_nodes(Rc::clone(&nodes[0]), short_node);
                new_parent = new_node_from_nodes(new_child, Rc::clone(&nodes[1]));
            }
            else if right_shorter
            {
                let new_child = new_node_from_nodes(short_node, Rc::clone(&nodes[1]));
                new_parent = new_node_from_nodes(Rc::clone(&nodes[0]), new_child);
            }
            else
            {
                let left_bvh_heuristic  = calculate_shared_bvh_heuristic(&nodes[0], &short_node);
                let right_bvh_heuristic = calculate_shared_bvh_heuristic(&nodes[1], &short_node);
                if left_bvh_heuristic < right_bvh_heuristic
                {
                    let new_child = new_node_from_nodes(Rc::clone(&nodes[0]), short_node);
                    new_parent = new_node_from_nodes(new_child, Rc::clone(&nodes[1]));
                }
                else
                {
                    let new_child = new_node_from_nodes(short_node, Rc::clone(&nodes[1]));
                    new_parent = new_node_from_nodes(Rc::clone(&nodes[0]), new_child);
                }
            }
        }
        else
        {
            unreachable!();
        };
    }
    else
    {
        return false;
    }
    *parent = new_parent;
    return true;
}
fn rebalance(parent : &mut NodeRef)
{
    let mut dirty = rebalance_internal(parent);
    while dirty
    {
        dirty = rebalance_internal(parent);
    }
}

fn insert_node(parent : &mut NodeRef, new_node : NodeRef)
{
    if parent.borrow().child.is_shape()
    {
        let old_node = Rc::clone(parent);
        *parent = new_node_from_nodes(old_node, new_node);
    }
    else
    {
        let new_parent;
        if let TreeChild::Nodes(nodes) = &parent.borrow().child
        {
            // FIXME: is this the right way to do this?
            let left_bvh_heuristic  = calculate_shared_bvh_heuristic(&nodes[0], &new_node);
            let right_bvh_heuristic = calculate_shared_bvh_heuristic(&nodes[1], &new_node);
            if left_bvh_heuristic < right_bvh_heuristic
            {
                let mut new_child = Rc::clone(&nodes[0]);
                insert_node(&mut new_child, new_node);
                new_parent = new_node_from_nodes(new_child, Rc::clone(&nodes[1]));
            }
            else
            {
                let mut new_child = Rc::clone(&nodes[1]);
                insert_node(&mut new_child, new_node);
                new_parent = new_node_from_nodes(Rc::clone(&nodes[0]), new_child);
            }
        }
        else
        {
            unreachable!();
        }
        *parent = new_parent;
        rebalance(parent)
    }
}
fn insert_shape(parent : &mut NodeRef, new_shape : PositionedShape)
{
    let new_node = new_node_from_shape(new_shape);
    insert_node(parent, new_node)
}

trait MyFormat {
    fn format(&self, indent : usize) -> String;
    fn dump_rects(&self) -> String;
}

impl MyFormat for NodeRef {
    fn format(&self, indent : usize) -> String
    {
        let mut ret = String::new();
        let borrowed = self.borrow();
        ret += &(" ".repeat(indent) + "{\n");
        ret += &format!("{}bounds: {:?}\n", " ".repeat(indent+4), borrowed.bounds);
        ret += &format!("{}leafs: {}\n", " ".repeat(indent+4), borrowed.leafs);
        ret += &format!("{}depth: {}\n", " ".repeat(indent+4), borrowed.depth);
        match &borrowed.child
        {
            TreeChild::Nodes(nodes) =>
            {
                ret += &(" ".repeat(indent+4) + "nodes:\n");
                ret += &(" ".repeat(indent+4) + "[\n");
                ret += &(nodes[0].format(indent+8) + "\n");
                ret += &(nodes[1].format(indent+8) + "\n");
                ret += &(" ".repeat(indent+4) + "]\n");
            }
            TreeChild::Shape(shape) =>
            {
                ret += &(" ".repeat(indent+4) + "shape\n");
            }
        }
        ret += &(" ".repeat(indent) + "}");
        ret
    }
    fn dump_rects(&self) -> String
    {
        let mut ret = String::new();
        let borrowed = self.borrow();
        ret += &format!("{}\n", borrowed.bounds.dump_rects(borrowed.depth));
        match &borrowed.child
        {
            TreeChild::Nodes(nodes) =>
            {
                ret += &format!("{}\n", nodes[0].dump_rects());
                ret += &format!("{}", nodes[1].dump_rects());
            }
            TreeChild::Shape(shape) =>
            {
                ret += &format!("{}", shape.borrow().aabb_positioned.dump_rects(0));
            }
        }
        ret
    }
}

#[derive(Debug)]
struct World {
    shape_counter : u64,
    shapes : BTreeMap<u64, ShapeRef>,
    static_tree : Option<NodeRef>,
    dynamic_tree : Option<NodeRef>,
}

impl World {
    fn new() -> World
    {
        World {
            shape_counter : 1,
            shapes : BTreeMap::new(),
            static_tree : None,
            dynamic_tree : None
        }
    }
    fn add_static_16px_box(&mut self, origin : Point)
    {
        let shape = Shape::Poly(Polygon{
            points : vec!(
                //Point::from(-8.0, -8.0),
                //Point::from(-8.0,  8.0),
                //Point::from( 8.0,  8.0),
                //Point::from( 8.0, -8.0),
                Point::from(0.0, 0.0),
                Point::from(0.0, 16.0),
                Point::from(16.0, 16.0),
                Point::from(16.0, 0.0),
            )
        });
        let mut positioned_shape = PositionedShape{
            aabb_raw : shape.calculate_raw_aabb(),
            aabb_positioned : shape.calculate_raw_aabb().translate(&origin),
            shape,
            origin,
            parent : Weak::new(),
            id : self.shape_counter,
            is_static : true,
        };
        self.shape_counter += 1;
        
        if let Some(ref mut tree) = self.static_tree
        {
            insert_shape(tree, positioned_shape);
        }
        else
        {
            self.static_tree = Some(new_node_from_shape(positioned_shape));
        };
    }
    fn format(&self) -> String
    {
        format!(
            "world:\n{{\n    statics:\n{}\n    dynamics:\n{}\n}}",
            self.static_tree .as_ref().map(|x| x.format(8)).unwrap_or("        (None)".to_string()),
            self.dynamic_tree.as_ref().map(|x| x.format(8)).unwrap_or("        (None)".to_string()),
        )
    }
    fn dump_rects(&self) -> String
    {
        format!(
            "\n{}\n{}",
            self.static_tree .as_ref().map(|x| x.dump_rects()).unwrap_or("".to_string()),
            self.dynamic_tree.as_ref().map(|x| x.dump_rects()).unwrap_or("".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_insertions()
    {
        let mut world = World::new();
        println!("adding first");
        let mut x = 16.0;
        let mut y = 16.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("adding second");
        x += 16.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("adding third");
        x += 16.0;
        y += 16.0;
        world.add_static_16px_box(Point::from(x, y));
        y += 16.0;
        world.add_static_16px_box(Point::from(x, y));
        y += 16.0;
        world.add_static_16px_box(Point::from(x, y));
        x += 48.0;
        y -= 16.0;
        world.add_static_16px_box(Point::from(x, y));
        x -= 32.0;
        y -= 32.0;
        x = 16.0;
        y = 32.0;
        world.add_static_16px_box(Point::from(x, y));
        y += 32.0;
        world.add_static_16px_box(Point::from(x, y));
        x = 96.0;
        y = 16.0;
        world.add_static_16px_box(Point::from(x, y));
        x -= 16.0;
        y += 16.0;
        world.add_static_16px_box(Point::from(x, y));
        x = 16.0;
        y = 96.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
    }
    #[test]
    fn test_float_min_max()
    {
        macro_rules! assert_eq_float {
            ( $a:expr, $b:expr ) => { assert_eq!($a.to_bits(), $b.to_bits()) }
        };
        macro_rules! assert_ne_float {
            ( $a:expr, $b:expr ) => { assert_ne!($a.to_bits(), $b.to_bits()) }
        };
        
        assert_eq_float!(float_max(0.0, -0.0), 0.0f64);
        assert_ne_float!(float_max(0.0, -0.0), -0.0f64);
        assert_eq_float!(float_min(0.0, -0.0), -0.0f64);
        assert_ne_float!(float_min(0.0, -0.0), 0.0f64);
        
        assert_eq_float!(float_max(1.0,  0.0), 1.0f64);
        assert_eq_float!(float_max(1.0, -1.0), 1.0f64);
        assert_eq_float!(float_max(0.0, -1.0), 0.0f64);
        assert_eq_float!(float_max(-0.0, -1.0), -0.0f64);
        
        assert_eq_float!(float_min(1.0,  -0.0),  -0.0f64);
        assert_eq_float!(float_min(1.0,  0.0),  0.0f64);
        assert_eq_float!(float_min(1.0, -1.0), -1.0f64);
        assert_eq_float!(float_min(0.0, -1.0), -1.0f64);
        
        assert_eq_float!(float_max(std::f64::INFINITY, std::f64::NEG_INFINITY), std::f64::INFINITY);
        assert_eq_float!(float_min(std::f64::INFINITY, std::f64::NEG_INFINITY), std::f64::NEG_INFINITY);
        
        assert_eq_float!(float_min(std::f64::INFINITY, std::f64::NEG_INFINITY), std::f64::NEG_INFINITY);
    }
}