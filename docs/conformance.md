# CNCF Conformance

This checklist follows official conformance requirements published by CNCF.[^source]

## Requirements

The requirements are grouped by their upstream SIG. Expand an area to view
its checklist; each requirement links to its official definition.

<details>
<summary>API Machinery</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [Priority and Fairness FlowSchema API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#priority-and-fairness-flowschema-api) | [ ] | |
| [Priority and Fairness PriorityLevelConfiguration API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#priority-and-fairness-prioritylevelconfiguration-api) | [ ] | |
| [Admission webhook, list mutating webhooks](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-list-mutating-webhooks) | [ ] | |
| [Admission webhook, list validating webhooks](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-list-validating-webhooks) | [ ] | |
| [Admission webhook, update mutating webhook](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-update-mutating-webhook) | [ ] | |
| [Admission webhook, update validating webhook](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-update-validating-webhook) | [ ] | |
| [Mutating Admission webhook, create and update mutating webhook configuration with matchConditions](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#mutating-admission-webhook-create-and-update-mutating-webhook-configuration-with-matchconditions) | [ ] | |
| [Validating Admission webhook, create and update validating webhook configuration with matchConditions](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#validating-admission-webhook-create-and-update-validating-webhook-configuration-with-matchconditions) | [ ] | |
| [Admission webhook, deny attach](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-deny-attach) | [ ] | |
| [Admission webhook, deny custom resource create and delete](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-deny-custom-resource-create-and-delete) | [ ] | |
| [Admission webhook, deny create](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-deny-create) | [ ] | |
| [Admission webhook, deny custom resource definition](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-deny-custom-resource-definition) | [ ] | |
| [Admission webhook, honor timeout](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-honor-timeout) | [ ] | |
| [Admission webhook, discovery document](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-discovery-document) | [ ] | |
| [Admission webhook, ordered mutation](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-ordered-mutation) | [ ] | |
| [Admission webhook, mutate custom resource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-mutate-custom-resource) | [ ] | |
| [Admission webhook, mutate custom resource with different stored version](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-mutate-custom-resource-with-different-stored-version) | [ ] | |
| [Admission webhook, mutate custom resource with pruning](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-mutate-custom-resource-with-pruning) | [ ] | |
| [Mutating Admission webhook, mutating webhook excluding object with specific name](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#mutating-admission-webhook-mutating-webhook-excluding-object-with-specific-name) | [ ] | |
| [Admission webhook, mutation with defaulting](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-mutation-with-defaulting) | [ ] | |
| [Admission webhook, admission control not allowed on webhook configuration objects](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-admission-control-not-allowed-on-webhook-configuration-objects) | [ ] | |
| [Mutating Admission webhook, reject mutating webhook configurations with invalid matchConditions](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#mutating-admission-webhook-reject-mutating-webhook-configurations-with-invalid-matchconditions) | [ ] | |
| [Validing Admission webhook, reject validating webhook configurations with invalid matchConditions](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#validing-admission-webhook-reject-validating-webhook-configurations-with-invalid-matchconditions) | [ ] | |
| [Admission webhook, fail closed](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#admission-webhook-fail-closed) | [ ] | |
| [Aggregated Discovery Interface](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#aggregated-discovery-interface) | [ ] | |
| [Aggregated Discovery Interface CRDs](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#aggregated-discovery-interface-crds) | [ ] | |
| [Aggregated Discovery Endpoint Accept Headers](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#aggregated-discovery-endpoint-accept-headers) | [ ] | |
| [Aggregated Discovery Endpoint Accept Headers CRDs](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#aggregated-discovery-endpoint-accept-headers-crds) | [ ] | |
| [aggregator-supports-the-sample-apiserver](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#aggregator-supports-the-sample-apiserver) | [ ] | |
| [Custom Resource Definition Conversion Webhook, convert mixed version list](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-conversion-webhook-convert-mixed-version-list) | [ ] | |
| [Custom Resource Definition Conversion Webhook, conversion custom resource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-conversion-webhook-conversion-custom-resource) | [ ] | |
| [Custom Resource Definition, watch](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-watch) | [ ] | |
| [Custom Resource Definition, create](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-create) | [ ] | |
| [Custom Resource Definition, status sub-resource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-status-sub-resource) | [ ] | |
| [Custom Resource Definition, list](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-list) | [ ] | |
| [Custom Resource Definition, defaulting](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-defaulting) | [ ] | |
| [Custom Resource Definition, discovery](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-discovery) | [ ] | |
| [custom-resource-definition-field-selectors-list-watch-register-informers](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-definition-field-selectors-list-watch-register-informers) | [ ] | |
| [Custom Resource OpenAPI Publish, stop serving version](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-stop-serving-version) | [ ] | |
| [Custom Resource OpenAPI Publish, version rename](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-version-rename) | [ ] | |
| [Custom Resource OpenAPI Publish, with x-kubernetes-preserve-unknown-fields at root](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-with-x-kubernetes-preserve-unknown-fields-at-root) | [ ] | |
| [Custom Resource OpenAPI Publish, with x-kubernetes-preserve-unknown-fields in embedded object](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-with-x-kubernetes-preserve-unknown-fields-in-embedded-object) | [ ] | |
| [Custom Resource OpenAPI Publish, with validation schema](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-with-validation-schema) | [ ] | |
| [Custom Resource OpenAPI Publish, with x-kubernetes-preserve-unknown-fields in object](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-with-x-kubernetes-preserve-unknown-fields-in-object) | [ ] | |
| [Custom Resource OpenAPI Publish, varying groups](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-varying-groups) | [ ] | |
| [Custom Resource OpenAPI Publish, varying kinds](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-varying-kinds) | [ ] | |
| [Custom Resource OpenAPI Publish, varying versions](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#custom-resource-openapi-publish-varying-versions) | [ ] | |
| [Discovery, confirm the groupVerion and a resourcefrom each apiGroup](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#discovery-confirm-the-groupverion-and-a-resourcefrom-each-apigroup) | [ ] | |
| [Discovery, confirm the PreferredVersion for each api group](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#discovery-confirm-the-preferredversion-for-each-api-group) | [ ] | |
| [Server side field validation, unknown fields CR no validation schema](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#server-side-field-validation-unknown-fields-cr-no-validation-schema) | [ ] | |
| [Server side field validation, valid CR with validation schema](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#server-side-field-validation-valid-cr-with-validation-schema) | [ ] | |
| [Server side field validation, unknown fields CR fails validation](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#server-side-field-validation-unknown-fields-cr-fails-validation) | [ ] | |
| [Server side field validation, CR duplicates](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#server-side-field-validation-cr-duplicates) | [ ] | |
| [Server side field validation, typed object](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#server-side-field-validation-typed-object) | [ ] | |
| [Server side field validation, unknown metadata](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#server-side-field-validation-unknown-metadata) | [ ] | |
| [Server side field validation, typed unknown metadata](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#server-side-field-validation-typed-unknown-metadata) | [ ] | |
| [Garbage Collector, delete deployment,  propagation policy background](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#garbage-collector-delete-deployment--propagation-policy-background) | [ ] | |
| [Garbage Collector, delete replication controller, propagation policy background](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#garbage-collector-delete-replication-controller-propagation-policy-background) | [ ] | |
| [Garbage Collector, delete replication controller, after owned pods](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#garbage-collector-delete-replication-controller-after-owned-pods) | [ ] | |
| [Garbage Collector, dependency cycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#garbage-collector-dependency-cycle) | [ ] | |
| [Garbage Collector, multiple owners](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#garbage-collector-multiple-owners) | [ ] | |
| [Garbage Collector, delete deployment, propagation policy orphan](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#garbage-collector-delete-deployment-propagation-policy-orphan) | [ ] | |
| [Garbage Collector, delete replication controller, propagation policy orphan](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#garbage-collector-delete-replication-controller-propagation-policy-orphan) | [ ] | |
| [Namespace, apply finalizer to a namespace](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#namespace-apply-finalizer-to-a-namespace) | [ ] | |
| [Namespace, apply update to a namespace](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#namespace-apply-update-to-a-namespace) | [ ] | |
| [Namespace, apply changes to a namespace status](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#namespace-apply-changes-to-a-namespace-status) | [ ] | |
| [namespace-deletion-removes-pods](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#namespace-deletion-removes-pods) | [ ] | |
| [namespace-deletion-removes-services](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#namespace-deletion-removes-services) | [ ] | |
| [Namespace patching](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#namespace-patching) | [ ] | |
| [Ordered Namespace Deletion](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ordered-namespace-deletion) | [ ] | |
| [ResourceQuota, apply changes to a ResourceQuota status](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-apply-changes-to-a-resourcequota-status) | [ ] | |
| [ResourceQuota, update and delete](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-update-and-delete) | [ ] | |
| [ResourceQuota, object count quota, configmap](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-object-count-quota-configmap) | [ ] | |
| [ResourceQuota, object count quota, pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-object-count-quota-pod) | [ ] | |
| [ResourceQuota, object count quota, replicaSet](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-object-count-quota-replicaset) | [ ] | |
| [ResourceQuota, object count quota, replicationController](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-object-count-quota-replicationcontroller) | [ ] | |
| [ResourceQuota, object count quota, secret](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-object-count-quota-secret) | [ ] | |
| [ResourceQuota, object count quota, service](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-object-count-quota-service) | [ ] | |
| [ResourceQuota, object count quota, resourcequotas](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-object-count-quota-resourcequotas) | [ ] | |
| [ResourceQuota, manage lifecycle of a ResourceQuota](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-manage-lifecycle-of-a-resourcequota) | [ ] | |
| [ResourceQuota, quota scope, BestEffort and NotBestEffort scope](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-quota-scope-besteffort-and-notbesteffort-scope) | [ ] | |
| [ResourceQuota, quota scope, Terminating and NotTerminating scope](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#resourcequota-quota-scope-terminating-and-notterminating-scope) | [ ] | |
| [API Chunking, server should return chunks of results for list calls](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#api-chunking-server-should-return-chunks-of-results-for-list-calls) | [ ] | |
| [API Chunking, server should support continue listing from the last key even if the original version has been compacted away](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#api-chunking-server-should-support-continue-listing-from-the-last-key-even-if-the-original-version-has-been-compacted-away) | [ ] | |
| [API metadata HTTP return](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#api-metadata-http-return) | [ ] | |
| [ValidatingAdmissionPolicy](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#validatingadmissionpolicy) | [ ] | |
| [ValidatingAdmissionPolicy API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#validatingadmissionpolicy-api) | [ ] | |
| [ValidatingadmissionPolicyBinding API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#validatingadmissionpolicybinding-api) | [ ] | |
| [ValidatingAdmissionPolicy](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#validatingadmissionpolicy-1) | [ ] | |
| [watch-configmaps-closed-and-restarted](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#watch-configmaps-closed-and-restarted) | [ ] | |
| [watch-configmaps-from-resource-version](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#watch-configmaps-from-resource-version) | [ ] | |
| [watch-configmaps-with-multiple-watchers](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#watch-configmaps-with-multiple-watchers) | [ ] | |
| [watch-configmaps-label-changed](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#watch-configmaps-label-changed) | [ ] | |
| [watch-consistency](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#watch-consistency) | [ ] | |
| [Confirm a server version](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#confirm-a-server-version) | [ ] | |

</details>

<details>
<summary>Applications</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [ControllerRevision, resource lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#controllerrevision-resource-lifecycle) | [ ] | |
| [CronJob Suspend](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#cronjob-suspend) | [ ] | |
| [CronJob ForbidConcurrent](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#cronjob-forbidconcurrent) | [ ] | |
| [CronJob ReplaceConcurrent](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#cronjob-replaceconcurrent) | [ ] | |
| [CronJob AllowConcurrent](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#cronjob-allowconcurrent) | [ ] | |
| [CronJob API Operations](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#cronjob-api-operations) | [ ] | |
| [DaemonSet, list and delete a collection of DaemonSets](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#daemonset-list-and-delete-a-collection-of-daemonsets) | [ ] | |
| [DaemonSet-FailedPodCreation](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#daemonset-failedpodcreation) | [ ] | |
| [DaemonSet-Rollback](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#daemonset-rollback) | [ ] | |
| [DaemonSet-NodeSelection](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#daemonset-nodeselection) | [ ] | |
| [DaemonSet-Creation](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#daemonset-creation) | [ ] | |
| [DaemonSet-RollingUpdate](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#daemonset-rollingupdate) | [ ] | |
| [DaemonSet, status sub-resource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#daemonset-status-sub-resource) | [ ] | |
| [Deployment, completes the scaling of a Deployment subresource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#deployment-completes-the-scaling-of-a-deployment-subresource) | [ ] | |
| [Deployment Recreate](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#deployment-recreate) | [ ] | |
| [Deployment RollingUpdate](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#deployment-rollingupdate) | [ ] | |
| [Deployment RevisionHistoryLimit](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#deployment-revisionhistorylimit) | [ ] | |
| [Deployment Proportional Scaling](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#deployment-proportional-scaling) | [ ] | |
| [Deployment Rollover](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#deployment-rollover) | [ ] | |
| [Deployment, completes the lifecycle of a Deployment](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#deployment-completes-the-lifecycle-of-a-deployment) | [ ] | |
| [Deployment, status sub-resource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#deployment-status-sub-resource) | [ ] | |
| [PodDisruptionBudget: list and delete collection](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#poddisruptionbudget-list-and-delete-collection) | [ ] | |
| [PodDisruptionBudget: block an eviction until the PDB is updated to allow it](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#poddisruptionbudget-block-an-eviction-until-the-pdb-is-updated-to-allow-it) | [ ] | |
| [PodDisruptionBudget: create, update, patch, and delete object](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#poddisruptionbudget-create-update-patch-and-delete-object) | [ ] | |
| [PodDisruptionBudget: Status updates](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#poddisruptionbudget-status-updates) | [ ] | |
| [PodDisruptionBudget: update and patch status](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#poddisruptionbudget-update-and-patch-status) | [ ] | |
| [Jobs, orphan pods, re-adoption](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#jobs-orphan-pods-re-adoption) | [ ] | |
| [Ensure pod failure policy allows to ignore failure matching on the DisruptionTarget condition](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ensure-pod-failure-policy-allows-to-ignore-failure-matching-on-the-disruptiontarget-condition) | [ ] | |
| [Verify Pod Failure policy allows to fail job early on exit code.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#verify-pod-failure-policy-allows-to-fail-job-early-on-exit-code) | [ ] | |
| [Jobs, apply changes to status](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#jobs-apply-changes-to-status) | [ ] | |
| [Ensure Pods of an Indexed Job get a unique index.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ensure-pods-of-an-indexed-job-get-a-unique-index) | [ ] | |
| [Jobs, active pods, graceful termination](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#jobs-active-pods-graceful-termination) | [ ] | |
| [Ensure that all indexes are executed for an indexed job with backoffLimitPerIndex despite some failing](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ensure-that-all-indexes-are-executed-for-an-indexed-job-with-backofflimitperindex-despite-some-failing) | [ ] | |
| [Jobs, manage lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#jobs-manage-lifecycle) | [ ] | |
| [Mark indexes as failed when the FailIndex action is matched in podFailurePolicy](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#mark-indexes-as-failed-when-the-failindex-action-is-matched-in-podfailurepolicy) | [ ] | |
| [Jobs, completion after task failure](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#jobs-completion-after-task-failure) | [ ] | |
| [Terminate job execution when the maxFailedIndexes is exceeded](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#terminate-job-execution-when-the-maxfailedindexes-is-exceeded) | [ ] | |
| [Ensure that job with successPolicy succeeded when all indexes succeeded](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ensure-that-job-with-successpolicy-succeeded-when-all-indexes-succeeded) | [ ] | |
| [Ensure that job with successPolicy succeededCount rule succeeded even when some indexes remain pending](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ensure-that-job-with-successpolicy-succeededcount-rule-succeeded-even-when-some-indexes-remain-pending) | [ ] | |
| [Ensure that job with successPolicy succeededIndexes rule succeeded even when some indexes remain pending](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ensure-that-job-with-successpolicy-succeededindexes-rule-succeeded-even-when-some-indexes-remain-pending) | [ ] | |
| [ReplicaSet, is created, Replaced and Patched](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replicaset-is-created-replaced-and-patched) | [ ] | |
| [ReplicaSet, completes the scaling of a ReplicaSet subresource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replicaset-completes-the-scaling-of-a-replicaset-subresource) | [ ] | |
| [Replica Set, adopt matching pods and release non matching pods](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replica-set-adopt-matching-pods-and-release-non-matching-pods) | [ ] | |
| [ReplicaSet, list and delete a collection of ReplicaSets](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replicaset-list-and-delete-a-collection-of-replicasets) | [ ] | |
| [Replica Set, run basic image](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replica-set-run-basic-image) | [ ] | |
| [ReplicaSet, status sub-resource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replicaset-status-sub-resource) | [ ] | |
| [Replication Controller, adopt matching pods](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replication-controller-adopt-matching-pods) | [ ] | |
| [Replication Controller, get and update ReplicationController scale](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replication-controller-get-and-update-replicationcontroller-scale) | [ ] | |
| [Replication Controller, release pods](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replication-controller-release-pods) | [ ] | |
| [Replication Controller, run basic image](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replication-controller-run-basic-image) | [ ] | |
| [Replication Controller, check for issues like exceeding allocated quota](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replication-controller-check-for-issues-like-exceeding-allocated-quota) | [ ] | |
| [Replication Controller, lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#replication-controller-lifecycle) | [ ] | |
| [StatefulSet, Burst Scaling](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#statefulset-burst-scaling) | [ ] | |
| [StatefulSet, Scaling](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#statefulset-scaling) | [ ] | |
| [StatefulSet, Recreate Failed Pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#statefulset-recreate-failed-pod) | [ ] | |
| [StatefulSet resource Replica scaling](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#statefulset-resource-replica-scaling) | [ ] | |
| [StatefulSet, list, patch and delete a collection of StatefulSets](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#statefulset-list-patch-and-delete-a-collection-of-statefulsets) | [ ] | |
| [StatefulSet, Rolling Update with Partition](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#statefulset-rolling-update-with-partition) | [ ] | |
| [StatefulSet, Rolling Update](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#statefulset-rolling-update) | [ ] | |
| [StatefulSet, status sub-resource](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#statefulset-status-sub-resource) | [ ] | |

</details>

<details>
<summary>Architecture</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [Conformance tests minimum number of nodes.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#conformance-tests-minimum-number-of-nodes) | [ ] | |

</details>

<details>
<summary>Authentication and Authorization</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [CertificateSigningRequest API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#certificatesigningrequest-api) | [ ] | |
| [OIDC Discovery (ServiceAccountIssuerDiscovery)](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#oidc-discovery-serviceaccountissuerdiscovery) | [ ] | |
| [Service account tokens auto mount optionally](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-account-tokens-auto-mount-optionally) | [ ] | |
| [ServiceAccount, create and review token](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#serviceaccount-create-and-review-token) | [ ] | |
| [RootCA ConfigMap test](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#rootca-configmap-test) | [ ] | |
| [Service Account Tokens Must AutoMount](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-account-tokens-must-automount) | [ ] | |
| [TokenRequestProjection should mount a projected volume with token using TokenRequest API.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#tokenrequestprojection-should-mount-a-projected-volume-with-token-using-tokenrequest-api) | [ ] | |
| [ServiceAccount lifecycle test](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#serviceaccount-lifecycle-test) | [ ] | |
| [ServiceAccount, update a ServiceAccount](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#serviceaccount-update-a-serviceaccount) | [ ] | |
| [SubjectReview, API Operations](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#subjectreview-api-operations) | [ ] | |

</details>

<details>
<summary>CLI Tools</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [kubectl, guestbook application](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-guestbook-application) | [ ] | |
| [kubectl, check version v1](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-check-version-v1) | [ ] | |
| [kubectl, cluster info](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-cluster-info) | [ ] | |
| [kubectl, describe pod or rc](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-describe-pod-or-rc) | [ ] | |
| [kubectl, diff Deployment](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-diff-deployment) | [ ] | |
| [kubectl, create service, replication controller](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-create-service-replication-controller) | [ ] | |
| [kubectl, label update](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-label-update) | [ ] | |
| [kubectl, patch to annotate](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-patch-to-annotate) | [ ] | |
| [kubectl, replace](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-replace) | [ ] | |
| [kubectl, run pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-run-pod) | [ ] | |
| [kubectl, server-side dry-run Pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-server-side-dry-run-pod) | [ ] | |
| [kubectl, version](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-version) | [ ] | |
| [kubectl, proxy socket](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-proxy-socket) | [ ] | |
| [kubectl, proxy port zero](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-proxy-port-zero) | [ ] | |
| [kubectl, replication controller](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-replication-controller) | [ ] | |
| [kubectl, scale replication controller](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-scale-replication-controller) | [ ] | |
| [kubectl, logs](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubectl-logs) | [ ] | |

</details>

<details>
<summary>Instrumentation</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [New Event resource lifecycle, testing a list of events](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#new-event-resource-lifecycle-testing-a-list-of-events) | [ ] | |
| [New Event resource lifecycle, testing a single event](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#new-event-resource-lifecycle-testing-a-single-event) | [ ] | |
| [Event, delete a collection](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#event-delete-a-collection) | [ ] | |
| [Event, manage lifecycle of an Event](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#event-manage-lifecycle-of-an-event) | [ ] | |

</details>

<details>
<summary>Networking</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [kubernetes.default Endpoints and EndpointSlices](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubernetesdefault-endpoints-and-endpointslices) | [ ] | |
| [Kubernetes Service](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubernetes-service) | [ ] | |
| [DNS, cluster](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#dns-cluster) | [ ] | |
| [DNS, for ExternalName Services](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#dns-for-externalname-services) | [ ] | |
| [DNS, resolve the hostname](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#dns-resolve-the-hostname) | [ ] | |
| [DNS, resolve the subdomain](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#dns-resolve-the-subdomain) | [ ] | |
| [DNS, services](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#dns-services) | [ ] | |
| [DNS, cluster](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#dns-cluster-1) | [ ] | |
| [DNS, PQDN for services](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#dns-pqdn-for-services) | [ ] | |
| [DNS, custom dnsConfig](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#dns-custom-dnsconfig) | [ ] | |
| [EndpointSlice, creation/deletion](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpointslice-creationdeletion) | [ ] | |
| [EndpointSlice, "empty" Service](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpointslice-empty-service) | [ ] | |
| [EndpointSlice, multiple IPs, multiple ports](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpointslice-multiple-ips-multiple-ports) | [ ] | |
| [EndpointSlice, single IP, multiple ports](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpointslice-single-ip-multiple-ports) | [ ] | |
| [EndpointSlice API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpointslice-api) | [ ] | |
| [EndpointSlice Mirroring](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpointslice-mirroring) | [ ] | |
| [Endpoint resource lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpoint-resource-lifecycle) | [ ] | |
| [Endpoints, creation/deletion](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpoints-creationdeletion) | [ ] | |
| [Endpoints, "empty" Service](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#endpoints-empty-service) | [ ] | |
| [Scheduling, HostPort matching and HostIP and Protocol not-matching](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#scheduling-hostport-matching-and-hostip-and-protocol-not-matching) | [ ] | |
| [Ingress API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ingress-api) | [ ] | |
| [IngressClass API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ingressclass-api) | [ ] | |
| [Networking, intra pod http](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#networking-intra-pod-http) | [ ] | |
| [Networking, intra pod udp](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#networking-intra-pod-udp) | [ ] | |
| [Networking, intra pod http, from node](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#networking-intra-pod-http-from-node) | [ ] | |
| [Networking, intra pod http, from node](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#networking-intra-pod-http-from-node-1) | [ ] | |
| [Proxy, validate Proxy responses](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#proxy-validate-proxy-responses) | [ ] | |
| [Proxy, validate ProxyWithPath responses](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#proxy-validate-proxywithpath-responses) | [ ] | |
| [Proxy through apiserver to a Service](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#proxy-through-apiserver-to-a-service) | [ ] | |
| [Service endpoint latency, thresholds](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-endpoint-latency-thresholds) | [ ] | |
| [IPAddress API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ipaddress-api) | [ ] | |
| [ServiceCIDR API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#servicecidr-api) | [ ] | |
| [Service, change type, ClusterIP to ExternalName](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-change-type-clusterip-to-externalname) | [ ] | |
| [Service, change type, ExternalName to ClusterIP](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-change-type-externalname-to-clusterip) | [ ] | |
| [Service, change type, ExternalName to NodePort](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-change-type-externalname-to-nodeport) | [ ] | |
| [Service, change type, NodePort to ExternalName](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-change-type-nodeport-to-externalname) | [ ] | |
| [Service, NodePort Service](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-nodeport-service) | [ ] | |
| [Service, NodePort type, session affinity to None](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-nodeport-type-session-affinity-to-none) | [ ] | |
| [Service, ClusterIP type, session affinity to None](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-clusterip-type-session-affinity-to-none) | [ ] | |
| [Service, complete ServiceStatus lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-complete-servicestatus-lifecycle) | [ ] | |
| [Service, deletes a collection of services](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-deletes-a-collection-of-services) | [ ] | |
| [Find Kubernetes Service in default Namespace](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#find-kubernetes-service-in-default-namespace) | [ ] | |
| [Service, NodePort type, session affinity to ClientIP](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-nodeport-type-session-affinity-to-clientip) | [ ] | |
| [Service, ClusterIP type, session affinity to ClientIP](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-clusterip-type-session-affinity-to-clientip) | [ ] | |
| [Service, endpoints](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-endpoints) | [ ] | |
| [Service, should serve endpoints on same port and different protocols.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-should-serve-endpoints-on-same-port-and-different-protocols) | [ ] | |
| [Service, endpoints with multiple ports](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#service-endpoints-with-multiple-ports) | [ ] | |

</details>

<details>
<summary>Node Behavior</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [ConfigMap, from environment field with various prefixes](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-from-environment-field-with-various-prefixes) | [ ] | |
| [ConfigMap, from environment field](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-from-environment-field) | [ ] | |
| [ConfigMap, from environment variables](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-from-environment-variables) | [ ] | |
| [ConfigMap, with empty-key](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-with-empty-key) | [ ] | |
| [ConfigMap lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-lifecycle) | [ ] | |
| [ConfigMap Update](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-update) | [ ] | |
| [Pod Lifecycle, post start exec hook](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-lifecycle-post-start-exec-hook) | [ ] | |
| [Pod Lifecycle, post start http hook](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-lifecycle-post-start-http-hook) | [ ] | |
| [Pod Lifecycle, prestop exec hook](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-lifecycle-prestop-exec-hook) | [ ] | |
| [Pod Lifecycle, prestop http hook](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-lifecycle-prestop-http-hook) | [ ] | |
| [Container Runtime, TerminationMessage, from log output of succeeding container](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#container-runtime-terminationmessage-from-log-output-of-succeeding-container) | [ ] | |
| [Container Runtime, TerminationMessage, from file of succeeding container](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#container-runtime-terminationmessage-from-file-of-succeeding-container) | [ ] | |
| [Container Runtime, TerminationMessage, from container's log output of failing container](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#container-runtime-terminationmessage-from-containers-log-output-of-failing-container) | [ ] | |
| [Container Runtime, TerminationMessagePath, non-root user and non-default path](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#container-runtime-terminationmessagepath-non-root-user-and-non-default-path) | [ ] | |
| [Container Runtime, Restart Policy, Pod Phases](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#container-runtime-restart-policy-pod-phases) | [ ] | |
| [Containers, with arguments](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#containers-with-arguments) | [ ] | |
| [Containers, with command](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#containers-with-command) | [ ] | |
| [Containers, with command and arguments](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#containers-with-command-and-arguments) | [ ] | |
| [Containers, without command and arguments](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#containers-without-command-and-arguments) | [ ] | |
| [DownwardAPI, environment for CPU and memory limits and requests](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-environment-for-cpu-and-memory-limits-and-requests) | [ ] | |
| [DownwardAPI, environment for default CPU and memory limits and requests](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-environment-for-default-cpu-and-memory-limits-and-requests) | [ ] | |
| [DownwardAPI, environment for host ip](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-environment-for-host-ip) | [ ] | |
| [DownwardAPI, environment for hostIPs](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-environment-for-hostips) | [ ] | |
| [DownwardAPI, environment for Pod UID](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-environment-for-pod-uid) | [ ] | |
| [DownwardAPI, environment for name, namespace and ip](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-environment-for-name-namespace-and-ip) | [ ] | |
| [Ephemeral Container, update ephemeral containers](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ephemeral-container-update-ephemeral-containers) | [ ] | |
| [Ephemeral Container Creation](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#ephemeral-container-creation) | [ ] | |
| [init-container-starts-app-restartalways-pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#init-container-starts-app-restartalways-pod) | [ ] | |
| [init-container-starts-app-restartnever-pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#init-container-starts-app-restartnever-pod) | [ ] | |
| [init-container-fails-stops-app-restartnever-pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#init-container-fails-stops-app-restartnever-pod) | [ ] | |
| [init-container-fails-stops-app-restartalways-pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#init-container-fails-stops-app-restartalways-pod) | [ ] | |
| [Kubelet, log output, default](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubelet-log-output-default-1) | [ ] | |
| [Kubelet, failed pod, delete](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubelet-failed-pod-delete) | [ ] | |
| [Kubelet, failed pod, terminated reason](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubelet-failed-pod-terminated-reason) | [ ] | |
| [Kubelet, pod with read only root file system](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubelet-pod-with-read-only-root-file-system) | [ ] | |
| [Kubelet, hostAliases](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubelet-hostaliases) | [ ] | |
| [Kubelet, managed etc hosts](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#kubelet-managed-etc-hosts) | [ ] | |
| [lease API should be available](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#lease-api-should-be-available) | [ ] | |
| [Pod Eviction, Toleration limits](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-eviction-toleration-limits) | [ ] | |
| [Taint, Pod Eviction on taint removal](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#taint-pod-eviction-on-taint-removal) | [ ] | |
| [Node, resource lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#node-resource-lifecycle) | [ ] | |
| [In-place Pod Resize, burtable pod with multiple containers and various operations](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#in-place-pod-resize-burtable-pod-with-multiple-containers-and-various-operations) | [ ] | |
| [In-place Pod Resize, burstable pod resized with equivalents](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#in-place-pod-resize-burstable-pod-resized-with-equivalents) | [ ] | |
| [In-place Pod Resize, guaranteed pods with multiple containers, net increase](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#in-place-pod-resize-guaranteed-pods-with-multiple-containers-net-increase) | [ ] | |
| [In-place Pod Resize, guaranteed pods with multiple containers, net decrease](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#in-place-pod-resize-guaranteed-pods-with-multiple-containers-net-decrease) | [ ] | |
| [In-place Pod Resize, guaranteed pods with multiple containers, various operations](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#in-place-pod-resize-guaranteed-pods-with-multiple-containers-various-operations) | [ ] | |
| [In-place Pod Resize, read and replace endpoints](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#in-place-pod-resize-read-and-replace-endpoints) | [ ] | |
| [PodTemplate, delete a collection](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#podtemplate-delete-a-collection) | [ ] | |
| [PodTemplate, replace](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#podtemplate-replace) | [ ] | |
| [PodTemplate lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#podtemplate-lifecycle) | [ ] | |
| [Pods Generation, graceful delete](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-generation-graceful-delete) | [ ] | |
| [Pods Generation, 500 updates](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-generation-500-updates) | [ ] | |
| [Pods Generation, updates](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-generation-updates) | [ ] | |
| [Pods, QOS](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-qos) | [ ] | |
| [Pods, ActiveDeadlineSeconds](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-activedeadlineseconds) | [ ] | |
| [Pods, lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-lifecycle) | [ ] | |
| [Pods, update](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-update) | [ ] | |
| [Pods, service environment variables](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-service-environment-variables) | [ ] | |
| [Pods, delete a collection](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-delete-a-collection) | [ ] | |
| [Pods, assigned hostip](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-assigned-hostip) | [ ] | |
| [Pods, patching status](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-patching-status) | [ ] | |
| [Pods, completes the lifecycle of a Pod and the PodStatus](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-completes-the-lifecycle-of-a-pod-and-the-podstatus) | [ ] | |
| [Pods, remote command execution over websocket](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-remote-command-execution-over-websocket) | [ ] | |
| [Pods, logs from websockets](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-logs-from-websockets) | [ ] | |
| [Pods, prestop hook](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pods-prestop-hook) | [ ] | |
| [Pod liveness probe, using http endpoint, failure](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-liveness-probe-using-http-endpoint-failure) | [ ] | |
| [Pod liveness probe, using grpc call, success](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-liveness-probe-using-grpc-call-success) | [ ] | |
| [Pod liveness probe, using local file, no restart](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-liveness-probe-using-local-file-no-restart) | [ ] | |
| [Pod liveness probe, using tcp socket, no restart](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-liveness-probe-using-tcp-socket-no-restart) | [ ] | |
| [Pod liveness probe, using http endpoint, restart](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-liveness-probe-using-http-endpoint-restart) | [ ] | |
| [Pod liveness probe, using grpc call, failure](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-liveness-probe-using-grpc-call-failure) | [ ] | |
| [Pod liveness probe, using local file, restart](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-liveness-probe-using-local-file-restart) | [ ] | |
| [Pod liveness probe, using http endpoint, multiple restarts (slow)](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-liveness-probe-using-http-endpoint-multiple-restarts-slow) | [ ] | |
| [Pod readiness probe, with initial delay](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-readiness-probe-with-initial-delay) | [ ] | |
| [Pod readiness probe, failure](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-readiness-probe-failure) | [ ] | |
| [Pod with the deleted RuntimeClass is rejected.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-with-the-deleted-runtimeclass-is-rejected) | [ ] | |
| [Pod with the non-existing RuntimeClass is rejected.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-with-the-non-existing-runtimeclass-is-rejected) | [ ] | |
| [RuntimeClass Overhead field must be respected.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#runtimeclass-overhead-field-must-be-respected) | [ ] | |
| [Can schedule a pod requesting existing RuntimeClass.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#can-schedule-a-pod-requesting-existing-runtimeclass) | [ ] | |
| [RuntimeClass API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#runtimeclass-api) | [ ] | |
| [Secrets, pod environment from source](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-pod-environment-from-source) | [ ] | |
| [Secrets, pod environment field](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-pod-environment-field) | [ ] | |
| [Secrets, pod environment from source](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-pod-environment-from-source-1) | [ ] | |
| [Secrets, with empty-key](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-with-empty-key) | [ ] | |
| [Secret patching](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secret-patching) | [ ] | |
| [Security Context, runAsUser=65534](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#security-context-runasuser65534) | [ ] | |
| [Security Context, privileged=false.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#security-context-privilegedfalse) | [ ] | |
| [Security Context, readOnlyRootFilesystem=false.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#security-context-readonlyrootfilesystemfalse) | [ ] | |
| [Security Context, test RunAsGroup at container level](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#security-context-test-runasgroup-at-container-level) | [ ] | |
| [Security Context, test RunAsGroup at pod level](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#security-context-test-runasgroup-at-pod-level) | [ ] | |
| [Security Context, allowPrivilegeEscalation=false.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#security-context-allowprivilegeescalationfalse) | [ ] | |
| [Sysctls, reject invalid sysctls](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#sysctls-reject-invalid-sysctls) | [ ] | |
| [Sysctl, test sysctls](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#sysctl-test-sysctls) | [ ] | |
| [Environment variables, expansion](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#environment-variables-expansion) | [ ] | |
| [Environment variables, command argument expansion](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#environment-variables-command-argument-expansion) | [ ] | |
| [Environment variables, command expansion](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#environment-variables-command-expansion) | [ ] | |
| [VolumeSubpathEnvExpansion, subpath expansion](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#volumesubpathenvexpansion-subpath-expansion) | [ ] | |
| [VolumeSubpathEnvExpansion, subpath with absolute path](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#volumesubpathenvexpansion-subpath-with-absolute-path) | [ ] | |
| [VolumeSubpathEnvExpansion, subpath with backticks](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#volumesubpathenvexpansion-subpath-with-backticks) | [ ] | |
| [VolumeSubpathEnvExpansion, subpath test writes](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#volumesubpathenvexpansion-subpath-test-writes) | [ ] | |
| [VolumeSubpathEnvExpansion, subpath ready from failed state](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#volumesubpathenvexpansion-subpath-ready-from-failed-state) | [ ] | |
| [CRUD operations for deviceclasses](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#crud-operations-for-deviceclasses) | [ ] | |
| [CRUD operations for resourceclaims](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#crud-operations-for-resourceclaims) | [ ] | |
| [CRUD operations for resourceclaimtemplates](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#crud-operations-for-resourceclaimtemplates) | [ ] | |
| [CRUD operations for resoureslices](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#crud-operations-for-resoureslices) | [ ] | |

</details>

<details>
<summary>Scheduling</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [LimitRange, resources](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#limitrange-resources) | [ ] | |
| [LimitRange, list, patch and delete a LimitRange by collection](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#limitrange-list-patch-and-delete-a-limitrange-by-collection) | [ ] | |
| [Scheduler, resource limits](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#scheduler-resource-limits) | [ ] | |
| [Scheduler, node selector matching](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#scheduler-node-selector-matching) | [ ] | |
| [Scheduler, node selector not matching](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#scheduler-node-selector-not-matching) | [ ] | |
| [Scheduling, HostPort and Protocol match, HostIPs different but one is default HostIP (0.0.0.0)](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#scheduling-hostport-and-protocol-match-hostips-different-but-one-is-default-hostip-0000) | [ ] | |
| [Pod preemption verification](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#pod-preemption-verification) | [ ] | |
| [Scheduler, Verify PriorityClass endpoints](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#scheduler-verify-priorityclass-endpoints) | [ ] | |
| [Scheduler, Basic Preemption](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#scheduler-basic-preemption) | [ ] | |
| [Scheduler, Preemption for critical pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#scheduler-preemption-for-critical-pod) | [ ] | |
| [Verify the DisruptionTarget condition is added to the preempted pod](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#verify-the-disruptiontarget-condition-is-added-to-the-preempted-pod) | [ ] | |

</details>

<details>
<summary>Storage</summary>

| Kubernetes requirement | Met | Notes |
| --- | :---: | --- |
| [CSIDriver, lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#csidriver-lifecycle) | [ ] | |
| [CSIInlineVolumes should support Pods with inline volumes](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#csiinlinevolumes-should-support-pods-with-inline-volumes) | [ ] | |
| [CSINode, lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#csinode-lifecycle) | [ ] | |
| [CSIStorageCapacity API](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#csistoragecapacity-api) | [ ] | |
| [ConfigMap Volume, text data, binary data](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-text-data-binary-data) | [ ] | |
| [ConfigMap Volume, create, update and delete](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-create-update-and-delete) | [ ] | |
| [ConfigMap Volume, without mapping](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-without-mapping) | [ ] | |
| [ConfigMap Volume, without mapping, non-root user](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-without-mapping-non-root-user) | [ ] | |
| [ConfigMap Volume, without mapping, volume mode set](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-without-mapping-volume-mode-set) | [ ] | |
| [ConfigMap Volume, with mapping](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-with-mapping) | [ ] | |
| [ConfigMap Volume, with mapping, volume mode set](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-with-mapping-volume-mode-set) | [ ] | |
| [ConfigMap Volume, with mapping, non-root user](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-with-mapping-non-root-user) | [ ] | |
| [ConfigMap Volume, multiple volume maps](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-multiple-volume-maps) | [ ] | |
| [ConfigMap Volume, immutability](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-immutability) | [ ] | |
| [ConfigMap Volume, update](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#configmap-volume-update) | [ ] | |
| [DownwardAPI volume, CPU limits](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-cpu-limits) | [ ] | |
| [DownwardAPI volume, CPU request](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-cpu-request) | [ ] | |
| [DownwardAPI volume, memory limits](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-memory-limits) | [ ] | |
| [DownwardAPI volume, memory request](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-memory-request) | [ ] | |
| [DownwardAPI volume, CPU limit, default node allocatable](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-cpu-limit-default-node-allocatable) | [ ] | |
| [DownwardAPI volume, memory limit, default node allocatable](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-memory-limit-default-node-allocatable) | [ ] | |
| [DownwardAPI volume, pod name](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-pod-name) | [ ] | |
| [DownwardAPI volume, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-volume-mode-0400) | [ ] | |
| [DownwardAPI volume, file mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-file-mode-0400) | [ ] | |
| [DownwardAPI volume, update annotations](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-update-annotations) | [ ] | |
| [DownwardAPI volume, update label](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#downwardapi-volume-update-label) | [ ] | |
| [EmptyDir, Shared volumes between containers](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-shared-volumes-between-containers) | [ ] | |
| [EmptyDir, medium default, volume mode 0644](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-default-volume-mode-0644) | [ ] | |
| [EmptyDir, medium memory, volume mode 0644, non-root user](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-memory-volume-mode-0644-non-root-user) | [ ] | |
| [EmptyDir, medium default, volume mode 0666](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-default-volume-mode-0666) | [ ] | |
| [EmptyDir, medium memory, volume mode 0666,, non-root user](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-memory-volume-mode-0666-non-root-user) | [ ] | |
| [EmptyDir, medium default, volume mode 0777](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-default-volume-mode-0777) | [ ] | |
| [EmptyDir, medium memory, volume mode 0777, non-root user](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-memory-volume-mode-0777-non-root-user) | [ ] | |
| [EmptyDir, medium default, volume mode 0644](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-default-volume-mode-0644-1) | [ ] | |
| [EmptyDir, medium memory, volume mode 0644](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-memory-volume-mode-0644) | [ ] | |
| [EmptyDir, medium default, volume mode 0666](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-default-volume-mode-0666-1) | [ ] | |
| [EmptyDir, medium memory, volume mode 0666](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-memory-volume-mode-0666) | [ ] | |
| [EmptyDir, medium default, volume mode 0777](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-default-volume-mode-0777-1) | [ ] | |
| [EmptyDir, medium memory, volume mode 0777](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-memory-volume-mode-0777) | [ ] | |
| [EmptyDir, medium default, volume mode default](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-default-volume-mode-default) | [ ] | |
| [EmptyDir, medium memory, volume mode default](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-medium-memory-volume-mode-default) | [ ] | |
| [EmptyDir Wrapper Volume, ConfigMap volumes, no race](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-wrapper-volume-configmap-volumes-no-race) | [ ] | |
| [EmptyDir Wrapper Volume, Secret and ConfigMap volumes, no conflict](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#emptydir-wrapper-volume-secret-and-configmap-volumes-no-conflict) | [ ] | |
| [PersistentVolumes(Claims), apply changes to a pv/pvc status](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#persistentvolumesclaims-apply-changes-to-a-pvpvc-status) | [ ] | |
| [PersistentVolumes(Claims), lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#persistentvolumesclaims-lifecycle) | [ ] | |
| [Projected Volume, multiple projections](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-multiple-projections) | [ ] | |
| [Projected Volume, ConfigMap, create, update and delete](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-create-update-and-delete) | [ ] | |
| [Projected Volume, ConfigMap, volume mode default](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-volume-mode-default) | [ ] | |
| [Projected Volume, ConfigMap, non-root user](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-non-root-user) | [ ] | |
| [Projected Volume, ConfigMap, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-volume-mode-0400) | [ ] | |
| [Projected Volume, ConfigMap, mapped](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-mapped) | [ ] | |
| [Projected Volume, ConfigMap, mapped, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-mapped-volume-mode-0400) | [ ] | |
| [Projected Volume, ConfigMap, mapped, non-root user](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-mapped-non-root-user) | [ ] | |
| [Projected Volume, ConfigMap, multiple volume paths](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-multiple-volume-paths) | [ ] | |
| [Projected Volume, ConfigMap, update](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-configmap-update) | [ ] | |
| [Projected Volume, DownwardAPI, CPU limits](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-cpu-limits) | [ ] | |
| [Projected Volume, DownwardAPI, CPU request](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-cpu-request) | [ ] | |
| [Projected Volume, DownwardAPI, memory limits](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-memory-limits) | [ ] | |
| [Projected Volume, DownwardAPI, memory request](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-memory-request) | [ ] | |
| [Projected Volume, DownwardAPI, CPU limit, node allocatable](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-cpu-limit-node-allocatable) | [ ] | |
| [Projected Volume, DownwardAPI, memory limit, node allocatable](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-memory-limit-node-allocatable) | [ ] | |
| [Projected Volume, DownwardAPI, pod name](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-pod-name) | [ ] | |
| [Projected Volume, DownwardAPI, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-volume-mode-0400) | [ ] | |
| [Projected Volume, DownwardAPI, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-volume-mode-0400-1) | [ ] | |
| [Projected Volume, DownwardAPI, update annotation](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-update-annotation) | [ ] | |
| [Projected Volume, DownwardAPI, update labels](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-downwardapi-update-labels) | [ ] | |
| [Projected Volume, Secrets, create, update delete](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-secrets-create-update-delete) | [ ] | |
| [Projected Volume, Secrets, volume mode default](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-secrets-volume-mode-default) | [ ] | |
| [Project Volume, Secrets, non-root, custom fsGroup](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#project-volume-secrets-non-root-custom-fsgroup) | [ ] | |
| [Projected Volume, Secrets, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-secrets-volume-mode-0400) | [ ] | |
| [Projected Volume, Secrets, mapped](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-secrets-mapped) | [ ] | |
| [Projected Volume, Secrets, mapped, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-secrets-mapped-volume-mode-0400) | [ ] | |
| [Projected Volume, Secrets, mapped, multiple paths](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#projected-volume-secrets-mapped-multiple-paths) | [ ] | |
| [Secrets Volume, create, update and delete](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-create-update-and-delete) | [ ] | |
| [Secrets Volume, volume mode default, secret with same name in different namespace](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-volume-mode-default-secret-with-same-name-in-different-namespace) | [ ] | |
| [Secrets Volume, default](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-default) | [ ] | |
| [Secrets Volume, volume mode 0440, fsGroup 1001 and uid 1000](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-volume-mode-0440-fsgroup-1001-and-uid-1000) | [ ] | |
| [Secrets Volume, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-volume-mode-0400) | [ ] | |
| [Secrets Volume, mapping](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-mapping) | [ ] | |
| [Secrets Volume, mapping, volume mode 0400](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-mapping-volume-mode-0400) | [ ] | |
| [Secrets Volume, mapping multiple volume paths](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-mapping-multiple-volume-paths) | [ ] | |
| [Secrets Volume, immutability](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#secrets-volume-immutability) | [ ] | |
| [StorageClass, lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#storageclass-lifecycle) | [ ] | |
| [SubPath: Reading content from a configmap volume.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#subpath-reading-content-from-a-configmap-volume) | [ ] | |
| [SubPath: Reading content from a configmap volume.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#subpath-reading-content-from-a-configmap-volume-1) | [ ] | |
| [SubPath: Reading content from a downwardAPI volume.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#subpath-reading-content-from-a-downwardapi-volume) | [ ] | |
| [SubPath: Reading content from a projected volume.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#subpath-reading-content-from-a-projected-volume) | [ ] | |
| [SubPath: Reading content from a secret volume.](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#subpath-reading-content-from-a-secret-volume) | [ ] | |
| [VolumeAttachment, apply changes to a volumeattachment status](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#volumeattachment-apply-changes-to-a-volumeattachment-status) | [ ] | |
| [VolumeAttachment, lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#volumeattachment-lifecycle) | [ ] | |
| [VolumeAttributesClass, lifecycle](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md#volumeattributesclass-lifecycle) | [ ] | |

</details>

[^source]: [CNCF conformance documentation](https://github.com/cncf/k8s-conformance/blob/master/docs/README.md),
    [Kubernetes 1.35 requirements](https://github.com/cncf/k8s-conformance/blob/master/docs/KubeConformance-1.35.md).
    Repeated names refer to separate upstream entries.
[^status]: This is a requirements checklist, not a certification claim. A blank
    Notes cell means no assessment has been recorded. Results added to this
    checklist must identify the diorama release or revision they describe.

[Back to documentation](../README.md#documentation)
