(function () {
"use strict";

angular.module('public')
.controller('SignupController', SignupController);

SignupController.$inject = ['menuCategories','MenuService'];
function SignupController(menuCategories, MenuService) {
  var reg = this;
  reg.menuCategories = menuCategories;
    // $ctrl.menuCategories = menuCategories;
  reg.submit = function () {
    console.log("SignupController submit");
    var promise = MenuService.getMenuItems();
        promise.then(function (response) {

          reg.menu2 = response.menu_items.filter(function (el) {
                  return  ( el.short_name.indexOf(reg.user.favorite) !== -1 );
                });
          //console.log(list.found);
          MenuService.setFavoritedish(reg.user.favorite);
        })
        .catch(function (error) {
          console.log("Something went terribly wrong.");
        });

    reg.completed = true;
  };
}

})();
