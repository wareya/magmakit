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
    fn dot(&self, other : &Point) -> f64
    {
        self.x*other.x + self.y*other.y
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
    fn times(self, scale : f64) -> Point
    {
        Point{x : self.x*scale, y : self.y*scale}
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

impl std::cmp::PartialEq for Point
{
    fn eq(&self, other : &Self) -> bool
    {
        self.x == other.x && self.y == other.y
    }
}

#[inline]
fn determinant_2x2(a : f64, b : f64, c : f64, d : f64) -> f64
{
    a*d - b*c
}

#[derive(Clone, Copy, Debug)]
struct Line {
    start : Point,
    end : Point
}
impl Line {
    fn intersect_infinite(start_1 : &Point, end_1 : &Point, start_2 : &Point, end_2 : &Point) -> Point
    {
        let x1 = start_1.x;
        let y1 = start_1.y;
        let x2 = end_1.x;
        let y2 = end_1.y;
        let x3 = start_2.x;
        let y3 = start_2.y;
        let x4 = end_2.x;
        let y4 = end_2.y;
        
        let a = determinant_2x2(x1, y1, x2, y2);
        let b = determinant_2x2(x3, y3, x4, y4);
        let c = determinant_2x2(x1 - x2, y1 - y2, x3 - x4, y3 - y4);
        
        let x = determinant_2x2(a, x1 - x2, b, x3 - x4) / c;
        let y = determinant_2x2(a, y1 - y2, b, y3 - y4) / c;
        
        Point{x, y}
    }
    fn intersect_finite(start_1 : &Point, end_1 : &Point, start_2 : &Point, end_2 : &Point) -> Option<Point>
    {
        let ret = Line::intersect_infinite(start_1, end_1, start_2, end_2);
        let left   = float_min(start_1.x, end_1.x);
        let top    = float_min(start_1.y, end_1.y);
        let right  = float_max(start_1.x, end_1.x);
        let bottom = float_max(start_1.y, end_1.y);
        
        if ret.x < left || ret.x > right || ret.y < top || ret.y > bottom
        {
            None
        }
        else
        {
            Some(ret)
        }
    }
    fn relative_dot_product(start_1 : &Point, end_1 : &Point, start_2 : &Point, end_2 : &Point) -> f64
    {
        end_1.sub(&start_1).dot(&end_2.sub(&start_2))
    }
    // polygons wind clockwise, and positive y is down - an edge from (0.0, 0.0) to (10.0, 0.0) has a normal of (0.0, -1.0)
    fn as_badnormal(start : &Point, end : &Point) -> Point
    {
        let mut temp = start.sub(&end);
        std::mem::swap(&mut temp.x, &mut temp.y);
        temp.x = -temp.x;
        temp
    }
    fn as_normal(start : &Point, end : &Point) -> Point
    {
        *Line::as_badnormal(start, end).normalize_mut()
    }
    fn get_interpolant(this : &Point, mid : &Point, other : &Point) -> f64
    {
        let abs_x_delta = (other.x - this.x).abs();
        let abs_y_delta = (other.y - this.y).abs();
        if abs_x_delta > abs_y_delta
        {
            (mid.x - this.x) / (other.x - this.x)
        }
        else
        {
            (mid.y - this.y) / (other.y - this.y)
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct AABB {
    minima : Point,
    maxima : Point
}

const FATTEN_AMOUNT : f64 = 16.0;
const NODE_FATTEN_AMOUNT : f64 = 0.0;

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
    fn translation_union(&self, add : &Point) -> AABB
    {
        AABB{minima : self.minima.minima(&self.minima.add(add)), maxima : self.maxima.maxima(&self.maxima.add(add))}
    }
    fn fatten(&self) -> AABB
    {
        AABB {
            minima : self.minima.sub(&Point{x:FATTEN_AMOUNT, y:FATTEN_AMOUNT}),
            maxima : self.maxima.add(&Point{x:FATTEN_AMOUNT, y:FATTEN_AMOUNT})
        }
    }
    fn fatten_node(&self) -> AABB
    {
        AABB {
            minima : self.minima.sub(&Point{x:NODE_FATTEN_AMOUNT, y:NODE_FATTEN_AMOUNT}),
            maxima : self.maxima.add(&Point{x:NODE_FATTEN_AMOUNT, y:NODE_FATTEN_AMOUNT})
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
    fn touches(&self, other : &AABB) -> bool
    {
        self .minima.x <= other.maxima.x &&
        self .minima.y <= other.maxima.y &&
        other.minima.x <= self .maxima.x &&
        other.minima.y <= self .maxima.y
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

impl PositionedShape {
    fn fattened_positioned_aabb(&self) -> AABB
    {
        if self.is_static
        {
            self.aabb_positioned.clone()
        }
        else
        {
            self.aabb_positioned.fatten()
        }
    }
}

fn trace(moving : &ShapeRef, fixed : &ShapeRef, motion : Point) -> Option<TraceInfo>
{
    let moving_borrowed = moving.borrow();
    let fixed_borrowed = fixed.borrow();
    
    let moving_aabb = moving_borrowed.aabb_positioned.translation_union(&motion);
    if !moving_aabb.touches(&fixed_borrowed.aabb_positioned)
    {
        return None;
    }
    let relative_position = moving_borrowed.origin.sub(&fixed_borrowed.origin);
    match (&moving_borrowed.shape, &fixed_borrowed.shape)
    {
        (Shape::Poly(moving_polygon), Shape::Poly(fixed_polygon)) =>
        {
            // translated by relative position (fixed polygon is treated as origin)
            let moving_points_translated = moving_polygon.points.iter().map(
                |point| point.add(&moving_borrowed.origin.sub(&fixed_borrowed.origin))
            ).collect::<Vec<_>>();
            // for testing lines from moving polygon into fixed polygon
            let moving_points_nudged = moving_points_translated.iter().map(
                |point| point.add(&motion)
            ).collect::<Vec<_>>();
            // for testing lines from fixed polygon into moving polygon
            let fixed_points_nudged = fixed_polygon.points.iter().map(
                |point| point.sub(&motion)
            ).collect::<Vec<_>>();
            
            let mut valid_normal = None;
            let mut closest_fraction = 1.0;
            let i_max = moving_polygon.points.len();
            let j_max = fixed_polygon.points.len();
            for i in 0..i_max
            {
                let start = &moving_points_translated[i];
                let end   = &moving_points_translated[(i+1) % i_max];
                let nudged_start = &moving_points_nudged[i];
                let nudged_end = &moving_points_nudged[(i+1) % i_max];
                for j in 0..j_max
                {
                    let other_start = &fixed_polygon.points[j];
                    let other_end = &fixed_polygon.points[(j+1) % j_max];
                    
                    // skip line pairs that are facing in the same general direction
                    if Line::relative_dot_product(start, end, other_start, other_end) >= 0.0
                    {
                        continue;
                    }
                    
                    let normal = Line::as_badnormal(other_start, other_end);
                    macro_rules! handle_case { ( $a:expr, $b:expr, $c:expr, $d:expr ) =>
                    {
                        if let Some(point) = Line::intersect_finite($a, $b, $c, $d)
                        {
                            let fraction = Line::get_interpolant($a, &point, $b);
                            if fraction < closest_fraction
                            {
                                closest_fraction = fraction;
                                valid_normal = Some(normal);
                            }
                        }
                    } };
                    // trace moving points into fixed line
                    handle_case!(start, nudged_start, other_start, other_end);
                    handle_case!(end  , nudged_end  , other_start, other_end);
                    
                    let nudged_other_start = &fixed_points_nudged[i];
                    let nudged_other_end = &fixed_points_nudged[(i+1) % i_max];
                    
                    // trace fixed points into moving line
                    handle_case!(other_start, nudged_other_start, start, end);
                    handle_case!(other_end  , nudged_other_end  , start, end);
                }
            }
            if let Some(normal) = valid_normal
            {
                Some(TraceInfo {
                    moving : Rc::clone(moving),
                    fixed : Rc::clone(fixed),
                    consumed_motion : motion.times(closest_fraction),
                    normal,
                    fraction : closest_fraction
                })
            }
            else
            {
                None
            }
        }
        (Shape::Circle(moving_circle), Shape::Circle(fixed_circle)) =>
        {
            let combined_radius = moving_circle.radius + fixed_circle.radius;
            let point_1 = 
                moving_borrowed.origin
                .sub(&fixed_borrowed.origin)
                .add(&moving_circle.origin_offset)
                .sub(&fixed_circle.origin_offset);
            // casting from point into expanded target circle at origin
            let point_2 = point_1.add(&motion);
            
            let x1 = point_1.x;
            let y1 = point_1.y;
            let x2 = point_2.x;
            let y2 = point_2.y;
            
            let dx = x2-x1;
            let dy = y2-y1;
            let dr2 = dx*dx + dy*dy;
            
            #[allow(non_snake_case)]
            let D = determinant_2x2(x1, x2, y1, y2);
            let semi_discriminant = combined_radius*combined_radius*dr2  - D*D;
            
            if semi_discriminant < 0.0
            {
                return None;
            }
            
            let ysign = if y2 > y1 { 1.0 } else { -1.0 };
            let discriminant = semi_discriminant.sqrt();
            let dxs = dx / dr2;
            let dys = dy / dr2;
            
            let x_base =  D * dys;
            let y_base = -D * dxs;
            
            let x_discrim = dxs * discriminant / dr2;
            let y_discrim = (dys * discriminant).abs() / dr2;
            
            let point_a = Point::from(x_base + ysign*x_discrim, y_base + y_discrim);
            let point_b = Point::from(x_base - ysign*x_discrim, y_base - y_discrim);
            
            let fraction_a = Line::get_interpolant(&point_1, &point_a, &point_2);
            let fraction_b = Line::get_interpolant(&point_1, &point_b, &point_2);
            
            let (normal, fraction) =
            if fraction_a < 0.0 && fraction_a > 1.0 && fraction_b < 0.0 && fraction_b > 1.0
            {
                return None;
            }
            else if fraction_a < fraction_b
            {
                (point_a.normalize(), fraction_a)
            }
            else
            {
                (point_b.normalize(), fraction_b)
            };
            
            Some(TraceInfo {
                moving : Rc::clone(moving),
                fixed : Rc::clone(fixed),
                consumed_motion : motion.times(fraction),
                normal,
                fraction
            })
        }
        _ => panic!("unimplemented circle-polygon trace")
    }
}

#[derive(Debug)]
struct TraceInfo {
    moving : ShapeRef,
    fixed : ShapeRef,
    consumed_motion : Point,
    normal : Point,
    fraction : f64
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
    is_root : bool,
}

impl TreeNode {
    fn inner_shallow_clone(&self) -> NodeRef
    {
        let ret = Rc::new(RefCell::new(
            TreeNode {
                bounds : self.bounds,
                child : match &self.child {
                    TreeChild::Nodes([left_node, right_node]) => TreeChild::Nodes([Rc::clone(&left_node), Rc::clone(&right_node)]),
                    TreeChild::Shape(shape) => TreeChild::Shape(Rc::clone(&shape))
                },
                parent : Weak::clone(&self.parent),
                leafs : self.leafs,
                depth : self.depth,
                is_root : self.is_root
            }
        ));
        let ret_borrowed = ret.borrow_mut();
        match &ret_borrowed.child
        {
            TreeChild::Nodes([left_node, right_node]) =>
            {
                println!("clone assigning child node parents; {} {}; assigning {}", left_node.ptr_format(), right_node.ptr_format(), ret.ptr_format());
                left_node.borrow_mut().parent = Rc::downgrade(&ret);
                right_node.borrow_mut().parent = Rc::downgrade(&ret);
            }
            TreeChild::Shape(shape) =>
            {
                println!("clone assigning child shape parent; {}; assigning {}", shape.ptr_format(), ret.ptr_format());
                shape.borrow_mut().parent = Rc::downgrade(&ret);
            }
        }
        drop(ret_borrowed);
        ret
    }
}

fn recalculate_aabb_shape(target : &ShapeRef)
{
    let mut target = target.borrow_mut();
    target.aabb_positioned = target.aabb_raw.translate(&target.origin);
    if let Some(new_target) = &target.parent.upgrade()
    {
        drop(target);
        recalculate_aabb_node(&new_target);
    }
}
fn recalculate_aabb_node(target : &NodeRef)
{
    let mut inner_target = target.borrow_mut();
    inner_target.bounds = match &inner_target.child
    {
        TreeChild::Nodes(nodes) => nodes[0].borrow().bounds.union(&nodes[1].borrow().bounds).fatten_node(),
        TreeChild::Shape(shape) => shape.borrow().fattened_positioned_aabb()
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

#[must_use]
fn new_node_from_nodes(left : NodeRef, right : NodeRef) -> NodeRef
{
    let left_borrowed = left.borrow();
    let right_borrowed = right.borrow();
    let bounds = left_borrowed.bounds.union(&right_borrowed.bounds).fatten_node();
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
        is_root : false
    }));
    
    if let TreeChild::Nodes(nodes) = &ret.borrow().child
    {
        println!("first assigning child node parents to {}", ret.ptr_format());
        nodes[0].borrow_mut().parent = Rc::downgrade(&ret);
        nodes[0].borrow_mut().is_root = false;
        nodes[1].borrow_mut().parent = Rc::downgrade(&ret);
        nodes[1].borrow_mut().is_root = false;
    }
    else
    {
        unreachable!();
    }
    
    ret
}
#[must_use]
fn new_node_from_shaperef(shaperef : ShapeRef) -> NodeRef
{
    let bounds = shaperef.borrow().fattened_positioned_aabb();
    
    let ret = Rc::new(RefCell::new(TreeNode {
        bounds,
        child : TreeChild::Shape(shaperef),
        parent : Weak::new(),
        leafs : 1,
        depth : 1,
        is_root : false
    }));
    
    if let TreeChild::Shape(shape) = &ret.borrow().child
    {

        println!("first assigning child shape parent to {}", ret.ptr_format());
        shape.borrow_mut().parent = Rc::downgrade(&ret);
    }
    else
    {
        unreachable!();
    }
    
    ret
}
#[must_use]
fn new_node_from_shape(shape : PositionedShape) -> NodeRef
{
    let shaperef = Rc::new(RefCell::new(shape));
    new_node_from_shaperef(shaperef)
}

fn assign_in_context(attached : &NodeRef, detached : NodeRef)
{
    println!("copying data into {} from {}", attached.ptr_format(), detached.ptr_format());
    let mut attached_borrowed = attached.borrow_mut();
    let detached_owned = Rc::try_unwrap(detached).unwrap().into_inner();
    attached_borrowed.bounds = detached_owned.bounds;
    attached_borrowed.child = detached_owned.child;
    // repoint child parents
    match &attached_borrowed.child
    {
        TreeChild::Nodes([left_node, right_node]) =>
        {
            println!("reassigning child node parents; {} {}; assigning {}", left_node.ptr_format(), right_node.ptr_format(), attached.ptr_format());
            left_node.borrow_mut().parent = Rc::downgrade(attached);
            right_node.borrow_mut().parent = Rc::downgrade(attached);
        }
        TreeChild::Shape(shape) =>
        {
            println!("reassigning child shape parent; {}; assigning {}", shape.ptr_format(), attached.ptr_format());
            shape.borrow_mut().parent = Rc::downgrade(attached);
        }
    }
    // do not copy parent
    attached_borrowed.leafs = detached_owned.leafs;
    attached_borrowed.depth = detached_owned.depth;
    // do not copy whether root
}

const ALLOWED_DEPTH_DISBALANCE : usize = 1;

#[must_use]
fn rebalance_internal(parent : &NodeRef) -> bool
{
    let parent_borrowed = parent.borrow();
    if let TreeChild::Nodes(nodes) = &parent_borrowed.child
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
        drop(parent_borrowed);
        if let TreeChild::Nodes(nodes) = &tall_node.borrow().child
        {
            // FIXME: is this the right way to do this?
            let left_shorter = nodes[0].borrow().depth+(ALLOWED_DEPTH_DISBALANCE-1) < nodes[1].borrow().depth;
            let right_shorter = nodes[0].borrow().depth > nodes[1].borrow().depth+(ALLOWED_DEPTH_DISBALANCE-1);
            if left_shorter
            {
                println!("rebalance point A");
                let new_child = new_node_from_nodes(Rc::clone(&nodes[0]), short_node);
                assign_in_context(parent, new_node_from_nodes(new_child, Rc::clone(&nodes[1])));
                return true;
            }
            else if right_shorter
            {
                println!("rebalance point B");
                let new_child = new_node_from_nodes(short_node, Rc::clone(&nodes[1]));
                assign_in_context(parent, new_node_from_nodes(Rc::clone(&nodes[0]), new_child));
                return true;
            }
            else
            {
                let left_bvh_heuristic  = calculate_shared_bvh_heuristic(&nodes[0], &short_node);
                let right_bvh_heuristic = calculate_shared_bvh_heuristic(&nodes[1], &short_node);
                if left_bvh_heuristic < right_bvh_heuristic
                {
                    println!("rebalance point C");
                    let new_child = new_node_from_nodes(Rc::clone(&nodes[0]), short_node);
                    assign_in_context(parent, new_node_from_nodes(new_child, Rc::clone(&nodes[1])));
                    return true;
                }
                else
                {
                    println!("rebalance point D");
                    let new_child = new_node_from_nodes(short_node, Rc::clone(&nodes[1]));
                    assign_in_context(parent, new_node_from_nodes(Rc::clone(&nodes[0]), new_child));
                    return true;
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
        false
    }
}
#[must_use]
fn rebalance(parent : &NodeRef)
{
    println!("rebalancing");
    let mut dirty = rebalance_internal(parent);
    while dirty
    {
        dirty = rebalance_internal(parent);
    }
}

fn remove_shape(shape : &ShapeRef)
{
    let interparent = shape.borrow().parent.upgrade().unwrap();
    let parent = interparent.borrow().parent.upgrade();
    if let Some(mut parent) = parent
    {
        println!("found parent");
        let mut keep = 
        if let TreeChild::Nodes(nodes) = &parent.borrow_mut().child
        {
            if Rc::ptr_eq(&nodes[0], &interparent)
            {
                println!("keeping right");
                Rc::clone(&nodes[1])
            }
            else
            {
                println!("keeping left");
                Rc::clone(&nodes[0])
            }
        }
        else
        {
            unreachable!();
        };
        println!("checking as keep");
        check_tree(&keep);
        assign_in_context(&parent, match keep.borrow().child
        {
            TreeChild::Nodes(ref nodes) =>
                new_node_from_nodes(Rc::clone(&nodes[0]), Rc::clone(&nodes[1])),
            TreeChild::Shape(ref shape) =>
                new_node_from_shaperef(Rc::clone(shape)),
        });
        println!("checking as parent");
        check_tree(&parent);
        recalculate(&parent);
        rebalance(&parent);
        
        let mut current = Rc::clone(&parent);
        loop
        {
            println!("checking self before finding parent");
            check_tree(&current);
            println!("trying to find and fix parent");
            let borrowed = current.borrow();
            if let Some(parent) = borrowed.parent.upgrade()
            {
                if borrowed.is_root
                {
                    panic!("thinks it's root, but it's not!");
                }
                let mut parent = Rc::clone(&parent);
                drop(borrowed);
                recalculate(&parent);
                println!("checking tree after recalculating");
                check_tree(&parent);
                rebalance(&parent);
                println!("checking tree after rebalancing");
                check_tree(&parent);
                println!("checks passed, was it root? {}", parent.borrow().is_root);
                current = parent;
            }
            else if !borrowed.is_root
            {
                panic!("failed to find root in remove_shape {:?}", borrowed);
            }
            else
            {
                println!("found root");
                println!("one more check on root (which has {} leafs and a depth of {})", borrowed.leafs, borrowed.depth);
                check_tree(&current);
                println!("passed, breaking");
                break;
            }
        }
    }
    else
    {
        panic!("failed to find parent in remove_shape");
    }
}

fn check_tree(parent : &NodeRef)
{
    //println!("checking node with depth {} and {} leafs", parent.borrow().depth, parent.borrow().leafs);
    if parent.borrow().parent.upgrade().is_none()
    {
        if !parent.borrow().is_root
        {
            panic!("tree node with no parent is not root; has {} leafs and a depth of {}", parent.borrow().leafs, parent.borrow().depth);
        }
        else if parent.borrow().is_root
        {
            println!("note: root pointer of {}", parent.ptr_format());
        }
    }
    else if parent.borrow().is_root
    {
        panic!("tree node with parent is root; has {} leafs and a depth of {}", parent.borrow().leafs, parent.borrow().depth);
    }
    match parent.borrow().child
    {
        TreeChild::Nodes(ref nodes) =>
        {
            assert!(nodes[0].borrow().is_root == false);
            assert!(nodes[1].borrow().is_root == false);
            if !Rc::ptr_eq(&nodes[0].borrow().parent.upgrade().unwrap(), parent)
            {
                panic!("left parent mismatch");
            }
            if !Rc::ptr_eq(&nodes[1].borrow().parent.upgrade().unwrap(), parent)
            {
                panic!("right parent mismatch");
            }
            check_tree(&nodes[0]);
            check_tree(&nodes[1]);
        },
        TreeChild::Shape(ref shape) =>
        {
            let was_parent = shape.borrow().parent.upgrade().unwrap();
            if !Rc::ptr_eq(&shape.borrow().parent.upgrade().unwrap(), parent)
            {
                panic!("shape parent mismatch; was {} should be {}", was_parent.ptr_format(), parent.ptr_format());
            }
        }
    }
}

fn recalculate(parent : &NodeRef)
{
    recalculate_aabb_node(&parent);
    let mut borrowed_parent = parent.borrow_mut();
    if let TreeChild::Nodes(nodes) = &borrowed_parent.child
    {
        let left_borrowed = nodes[0].borrow();
        let right_borrowed = nodes[1].borrow();
        let leafs = left_borrowed.leafs + right_borrowed.leafs;
        let depth = 1 + std::cmp::max(left_borrowed.depth, right_borrowed.depth);
        if !Rc::ptr_eq(&left_borrowed.parent.upgrade().unwrap(), parent)
        {
            println!("left out of sync in recalculate");
        }
        if !Rc::ptr_eq(&right_borrowed.parent.upgrade().unwrap(), parent)
        {
            println!("right out of sync in recalculate");
        }
        drop(left_borrowed);
        drop(right_borrowed);
        borrowed_parent.leafs = leafs;
        borrowed_parent.depth = depth;
    }
    else
    {
        borrowed_parent.leafs = 1;
        borrowed_parent.depth = 1;
    }
}

#[must_use]
fn insert_node(parent : &NodeRef, new_node : NodeRef)
{
    let was_root = parent.borrow().is_root;
    if parent.borrow().child.is_shape()
    {
        println!("insert point A");
        let new_parent = new_node_from_nodes(parent.borrow().inner_shallow_clone(), new_node);
        assign_in_context(parent, new_parent);
    }
    else
    {
        let parent_borrowed = parent.borrow();
        if let TreeChild::Nodes([left_node, right_node]) = &parent_borrowed.child
        {
            // FIXME: is this the right way to do this?
            let center_bvh_heuristic = parent.borrow().bounds.bvh_heuristic(); // the field we would be breaking up
            let left_bvh_heuristic  = calculate_shared_bvh_heuristic(left_node, &new_node);
            let right_bvh_heuristic = calculate_shared_bvh_heuristic(right_node, &new_node);
            if left_bvh_heuristic < right_bvh_heuristic && left_bvh_heuristic < center_bvh_heuristic
            {
                println!("insert point B");
                let new_child = insert_node(left_node, new_node);
                let new_parent = new_node_from_nodes(Rc::clone(left_node), Rc::clone(right_node));
                drop(parent_borrowed);
                assign_in_context(parent, new_parent);
                rebalance(&parent)
            }
            else if right_bvh_heuristic < center_bvh_heuristic
            {
                println!("insert point C");
                let new_child = insert_node(right_node, new_node);
                let new_parent = new_node_from_nodes(Rc::clone(left_node), Rc::clone(right_node));
                drop(parent_borrowed);
                assign_in_context(parent, new_parent);
                rebalance(&parent)
            }
            else
            {
                println!("insert point D");
                let new_parent = new_node_from_nodes(parent.borrow().inner_shallow_clone(), new_node);
                drop(parent_borrowed);
                assign_in_context(parent, new_parent);
                rebalance(&parent)
            }
        }
        else
        {
            unreachable!();
        }
    }
}
fn insert_shape(parent : &NodeRef, new_shape : ShapeRef)
{
    let new_node = new_node_from_shaperef(new_shape);
    insert_node(parent, new_node)
}

trait MyFormat {
    fn format(&self, indent : usize) -> String;
    fn dump_rects(&self) -> String;
}

trait BadFormat {
    fn ptr_format(&self) -> String;
}
impl BadFormat for NodeRef {
    fn ptr_format(&self) -> String
    {
        let ptr_current = Rc::into_raw(Rc::clone(self));
        let ret = format!("{:?}", ptr_current);
        unsafe
        {
            Rc::from_raw(ptr_current);
        }
        ret
    }
}
impl BadFormat for ShapeRef {
    fn ptr_format(&self) -> String
    {
        let ptr_current = Rc::into_raw(Rc::clone(self));
        let ret = format!("{:?}", ptr_current);
        unsafe
        {
            Rc::from_raw(ptr_current);
        }
        ret
    }
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

fn eventual_parent(node : &Rc<RefCell<TreeNode>>) -> Rc<RefCell<TreeNode>>
{
    if let Some(parent) = node.borrow().parent.upgrade()
    {
        eventual_parent(&parent)
    }
    else
    {
        Rc::clone(node)
    }
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
    fn remove_shape(&mut self, shapenum : u64)
    {
        if let Some(shape) = self.shapes.remove(&shapenum)
        {
            let parent = shape.borrow().parent.upgrade().unwrap();
            let superparent = eventual_parent(&parent);
            remove_shape(&shape);
            let new_superparent = eventual_parent(&parent);
        }
        else
        {
            panic!("failed to find shape to remove");
        }
        check_tree(self.static_tree.as_ref().unwrap());
    }
    fn move_shape(&mut self, shapenum : u64, newcoords : Point)
    {
        if let Some(shape) = self.shapes.remove(&shapenum)
        {
            println!("moving {}", shapenum);
            let mut shape_borrowed = shape.borrow_mut();
            let parent = shape_borrowed.parent.upgrade().unwrap();
            shape_borrowed.origin = newcoords;
            shape_borrowed.aabb_positioned = shape_borrowed.aabb_raw.translate(&newcoords);
            drop(shape_borrowed);
            
            if !shape.borrow().aabb_positioned.contained_by(&parent.borrow().bounds)
            {
                remove_shape(&shape);
                let mut target = eventual_parent(&parent);
                insert_shape(&mut target, Rc::clone(&shape));
                recalculate_aabb_shape(&shape);
            }
        }
        check_tree(self.static_tree.as_ref().unwrap());
    }
    fn add_static_16px_box(&mut self, origin : Point) -> u64
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
        let current_id = self.shape_counter;
        self.shape_counter += 1;
        let aabb_raw = shape.calculate_raw_aabb();
        let mut positioned_shape = PositionedShape{
            aabb_positioned : shape.calculate_raw_aabb().translate(&origin),
            aabb_raw,
            shape,
            origin,
            parent : Weak::new(),
            id : current_id,
            is_static : true,
        };
        let shaperef = Rc::new(RefCell::new(positioned_shape));
        self.shapes.insert(current_id, Rc::clone(&shaperef));
        
        if let Some(tree) = &self.static_tree
        {
            insert_shape(&tree, shaperef);
        }
        else
        {
            self.static_tree = Some(new_node_from_shaperef(shaperef));
            self.static_tree.as_ref().unwrap().borrow_mut().is_root = true;
        };
        check_tree(self.static_tree.as_ref().unwrap());
        current_id
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
        let mut x = 16.0;
        let mut y = 16.0;
        let first = world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        x += 16.0;
        let second = world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        x += 16.0;
        y += 16.0;
        let third = world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        y += 16.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        y += 16.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        x += 48.0;
        y -= 16.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        x -= 32.0;
        y -= 32.0;
        x = 16.0;
        y = 32.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        y += 32.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        x = 96.0;
        y = 16.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        x -= 16.0;
        y += 16.0;
        world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        x = 16.0;
        y = 96.0;
        let last = world.add_static_16px_box(Point::from(x, y));
        println!("{}", world.dump_rects());
        println!("leafs: {}", world.static_tree.as_ref().unwrap().borrow().leafs);
        assert!(world.static_tree.as_ref().unwrap().borrow().leafs == 11);
        println!("---------------removing the last rect");
        world.remove_shape(last);
        println!("{}", world.dump_rects());
        println!("leafs: {}", world.static_tree.as_ref().unwrap().borrow().leafs);
        assert!(world.static_tree.as_ref().unwrap().borrow().leafs == 10);
        println!("---------------removing the first rect");
        world.remove_shape(first);
        println!("{}", world.dump_rects());
        println!("leafs: {}", world.static_tree.as_ref().unwrap().borrow().leafs);
        assert!(world.static_tree.as_ref().unwrap().borrow().leafs == 9);
        
        println!("---------------removing the second rect");
        world.remove_shape(second);
        println!("{}", world.dump_rects());
        println!("leafs: {}", world.static_tree.as_ref().unwrap().borrow().leafs);
        assert!(world.static_tree.as_ref().unwrap().borrow().leafs == 8);
        
        println!("---------------moving the third rect");
        world.move_shape(third, Point::from(50.0, 20.0));
        println!("{}", world.dump_rects());
        println!("leafs: {}", world.static_tree.as_ref().unwrap().borrow().leafs);
        assert!(world.static_tree.as_ref().unwrap().borrow().leafs == 8);
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
    #[test]
    fn test_line_stuff()
    {
        let a_start = Point::from(0.0, 0.0);
        let a_end = Point::from(1.0, 0.5);
        let b_start = Point::from(0.8, 0.0);
        let b_end = Point::from(0.4, 0.8);
        let c_start = Point::from(0.0, 0.0);
        let c_end = Point::from(0.5, 1.1);
        println!("{:?}", Line::intersect_infinite(&a_start, &a_end, &b_start, &b_end));
        println!("{:?}", Line::intersect_infinite(&b_start, &b_end, &c_start, &c_end));
        println!("{:?}", Line::intersect_finite(&b_start, &b_end, &c_start, &c_end));
        
        assert!(Line::as_normal(&Point::from(0.0, 0.0), &Point::from(10.0, 0.0)) == Point::from(0.0, -1.0));
        // right and down
        // normal right and up
        let n_2 = Line::as_normal(&Point::from(0.0, 0.0), &Point::from(1.0, 1.0));
        println!("{:?}", n_2);
        assert!(n_2.x.signum() == 1.0);
        assert!(n_2.y.signum() == -1.0);
        // right and up
        // normal left and up
        let n_2 = Line::as_normal(&Point::from(0.0, 0.0), &Point::from(1.0, -1.0));
        println!("{:?}", n_2);
        assert!(n_2.x.signum() == -1.0);
        assert!(n_2.y.signum() == -1.0);
        // left and up
        // normal left and down
        let n_2 = Line::as_normal(&Point::from(0.0, 0.0), &Point::from(-1.0, -1.0));
        println!("{:?}", n_2);
        assert!(n_2.x.signum() == -1.0);
        assert!(n_2.y.signum() == 1.0);
        // left and down
        // normal right and down
        let n_2 = Line::as_normal(&Point::from(0.0, 0.0), &Point::from(-1.0, 1.0));
        println!("{:?}", n_2);
        assert!(n_2.x.signum() == 1.0);
        assert!(n_2.y.signum() == 1.0);
    }
}