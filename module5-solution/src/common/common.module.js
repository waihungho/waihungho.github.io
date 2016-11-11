(function() {
"use strict";

angular.module('common', [])
.constant('ApiPath', 'https://secret-shore-75416.herokuapp.com')
.config(config);

config.$inject = ['$httpProvider'];
function config($httpProvider) {
  $httpProvider.interceptors.push('loadingHttpInterceptor');
}

})();
