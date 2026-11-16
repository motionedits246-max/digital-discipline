//! Logical inner sizes. Small/high-DPI displays may cap the workspace floor to usable space.
#[derive(Debug,PartialEq)]
pub struct Layout {pub width:f64,pub height:f64,pub min_width:f64,pub min_height:f64,pub workspace:bool}
pub fn layout(mode:&str,work_width:f64,work_height:f64)->Option<Layout>{
 let workspace=matches!(mode,"full"|"popout");
 let (width,height,min_width,min_height)=match mode{
  "pill"=>(280.0,56.0,260.0,52.0),"quick"=>(410.0,430.0,410.0,430.0),
  "full"|"popout"=>(1220.0,790.0,960.0,640.0),_=>return None,
 };
 let (frame_w,frame_h)=if mode=="popout"{(32.0,56.0)}else{(0.0,0.0)};
 let available_w=(work_width-16.0-frame_w).max(1.0);
 let available_h=(work_height-16.0-frame_h).max(1.0);
 Some(Layout{width:f64::min(width,available_w),height:f64::min(height,available_h),min_width:f64::min(min_width,available_w),min_height:f64::min(min_height,available_h),workspace})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn workspace_has_a_real_minimum_and_compact_modes_restore_their_own_limits(){
  let full=layout("full",1920.0,1040.0).unwrap();assert_eq!((full.min_width,full.min_height),(960.0,640.0));assert_eq!((full.width,full.height),(1220.0,790.0));
  let pill=layout("pill",1920.0,1040.0).unwrap();assert_eq!((pill.min_width,pill.min_height),(260.0,52.0));assert!(!pill.workspace);
  let quick=layout("quick",1920.0,1040.0).unwrap();assert_eq!((quick.width,quick.height),(410.0,430.0));
 }
 #[test]fn scaled_small_displays_never_force_the_floor_beyond_available_space(){
  for scale in [1.0,1.25,1.5,2.0]{for mode in ["full","popout","quick"]{let w=1366.0/scale;let h=728.0/scale;let l=layout(mode,w,h).unwrap();assert!(l.width<=w&&l.height<=h);assert!(l.min_width<=l.width&&l.min_height<=l.height);}}
 }
 #[test]fn unsupported_modes_are_rejected(){assert!(layout("other",1920.0,1040.0).is_none());}
}
